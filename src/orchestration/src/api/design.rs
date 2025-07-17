//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//

use crate::{
    actions::invoke,
    common::{orch_tag::OrchestrationTag, tag::Tag, DesignConfig},
    prelude::InvokeResult,
    program::{Program, ProgramBuilder},
    program_database::ProgramDatabase,
};
use ::core::{future::Future, ops::Deref};

use ::core::fmt::Debug;
use std::sync::{Arc, Mutex};

use foundation::{containers::growable_vec::GrowableVec, prelude::CommonErrors};

pub type ProgramTag = Tag;
pub type DesignTag = Tag;

/// Provides [`DesignConfig`] with is bounded to the `Design` instance.
pub struct DesignConfigBounded(DesignConfig);

impl Deref for DesignConfigBounded {
    type Target = DesignConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

///
/// Design is a container for Application developer to register all it's components (functions, events, conditions, etc.)
/// and orchestrations (programs) in `config-by-code` approach.  If `config-by-file` is used, user does not need to use
/// [`Design::add_program`] since it will be loaded from the file. Read more in [`crate::api::Orchestration`].
///
pub struct Design {
    id: DesignTag,
    params: DesignConfig, // TODO: probably remove when we store it in ProgramDatabase
    pub(crate) db: ProgramDatabase,
    programs: GrowableVec<ProgramData>,
}

impl Debug for Design {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Design").field("id", &self.id).finish()
    }
}

impl Design {
    /// Creates a new `Design` instance with the given identifier and configuration `parameters`.
    pub fn new(id: DesignTag, params: DesignConfig) -> Self {
        const DEFAULT_PROGRAMS_CNT: usize = 1;
        Design {
            id,
            params,
            db: ProgramDatabase::new(params),
            programs: GrowableVec::new(DEFAULT_PROGRAMS_CNT),
        }
    }

    /// Returns the configuration parameters for this design.
    pub fn get_config(&self) -> DesignConfigBounded {
        DesignConfigBounded(self.params)
    }

    /// Returns the unique identifier for this design.
    pub fn id(&self) -> Tag {
        self.id
    }

    /// Registers a function as an invoke action.
    pub fn register_invoke_fn(&self, tag: Tag, action: invoke::InvokeFunctionType) -> Result<OrchestrationTag, CommonErrors> {
        self.db.register_invoke_fn(tag, action)
    }

    /// Registers an async function as an invoke action
    pub fn register_invoke_async<A, F>(&self, tag: Tag, action: A) -> Result<OrchestrationTag, CommonErrors>
    where
        A: Fn() -> F + 'static + Send + Clone,
        F: Future<Output = InvokeResult> + 'static + Send,
    {
        self.db.register_invoke_async(tag, action)
    }

    /// Registers a method on an object as an invoke action.
    pub fn register_invoke_method<T: 'static + Send>(
        &self,
        tag: Tag,
        object: Arc<Mutex<T>>,
        method: fn(&mut T) -> InvokeResult,
    ) -> Result<OrchestrationTag, CommonErrors> {
        self.db.register_invoke_method(tag, object, method)
    }

    /// Registers an async method on an object as an invoke action.
    pub fn register_invoke_method_async<T, M, F>(&self, tag: Tag, object: Arc<Mutex<T>>, method: M) -> Result<OrchestrationTag, CommonErrors>
    where
        T: 'static + Send,
        M: Fn(Arc<Mutex<T>>) -> F + 'static + Send + Clone,
        F: Future<Output = InvokeResult> + 'static + Send,
    {
        self.db.register_invoke_method_async(tag, object, method)
    }

    /// Registers an event in the design and returns an [`OrchestrationTag`] that can be used to reference this event in programs.
    pub fn register_event(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        self.db.register_event(tag)
    }

    /// Registers a shutdown event in the design.
    pub fn register_shutdown_event(&mut self, tag: Tag) -> Result<(), CommonErrors> {
        self.db.register_shutdown_event(tag)
    }

    /// Fetches an [`OrchestrationTag`] for a given tag, which can be used to reference the orchestration in programs.
    pub fn get_orchestration_tag(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        self.db.get_orchestration_tag(tag)
    }

    /// Adds a program to the design. The program is created using the provided closure, which receives a mutable reference to the design.
    pub fn add_program<F>(&mut self, name: &'static str, program_creator: F)
    where
        F: FnOnce(&mut Self, &mut ProgramBuilder) -> Result<(), CommonErrors> + 'static,
    {
        self.programs.push(ProgramData::new(name, Box::new(program_creator)));
    }

    pub(crate) fn has_any_programs(&self) -> bool {
        !self.programs.is_empty()
    }

    pub(super) fn get_programs(mut self, mut container: GrowableVec<Program>) -> Result<GrowableVec<Program>, CommonErrors> {
        while let Some(program_data) = self.programs.pop() {
            let mut builder = ProgramBuilder::new(program_data.0);
            (program_data.1)(&mut self, &mut builder)?;
            container.push(builder.build(&mut self)?);
        }

        Ok(container)
    }
}

type ProgramBuilderFn = Box<dyn FnOnce(&mut Design, &mut ProgramBuilder) -> Result<(), CommonErrors>>;

#[allow(dead_code)]
pub(super) struct ProgramData(&'static str, ProgramBuilderFn);

impl ProgramData {
    pub(super) fn new(name: &'static str, program: ProgramBuilderFn) -> Self {
        Self(name, program)
    }
}

#[cfg(test)]

mod tests {
    // Tests are disabled in Miri due to limitations of using OS calls that are done in Iceroxy2 backend.
    // Currently we do not have any constructor that can inject IPC provider (subject to change in the near future).

    use crate::actions::action::UserErrValue;

    use super::*;

    #[test]
    fn design_creation() {
        let id = Tag::from_str_static("design1");
        let params = DesignConfig::default();

        let design = Design::new(id, params.clone());

        assert_eq!(design.id(), id);
        assert_eq!(*design.get_config(), params);
    }

    fn action() -> Result<(), UserErrValue> {
        Ok(())
    }

    #[test]
    fn register_invoke_fn_success() {
        let id = Tag::from_str_static("design1");
        let params = DesignConfig::default();
        let design = Design::new(id, params);

        let tag = Tag::from_str_static("invoke_fn");

        let result = design.register_invoke_fn(tag.clone(), action);

        assert!(result.is_ok());
        let orchestration_tag = result.unwrap();
        assert_eq!(*orchestration_tag.tag(), tag);
    }

    #[test]
    fn register_invoke_fn_duplicate() {
        let id = Tag::from_str_static("design1");
        let params = DesignConfig::default();
        let design = Design::new(id, params);

        let tag = Tag::from_str_static("invoke_fn");

        // Register the function once
        let result = design.register_invoke_fn(tag.clone(), action.clone());
        assert!(result.is_ok());

        // Attempt to register the same function again
        let duplicate_result = design.register_invoke_fn(tag.clone(), action);
        assert!(duplicate_result.is_err());
        assert_eq!(duplicate_result.unwrap_err(), CommonErrors::AlreadyDone);
    }

    #[test]
    fn get_orchestration_tag_success() {
        let id = Tag::from_str_static("design1");
        let params = DesignConfig::default();
        let design = Design::new(id, params);

        let tag = Tag::from_str_static("orchestration_tag");

        // Register the function
        let _ = design.register_invoke_fn(tag.clone(), action);

        // Retrieve the orchestration tag
        let orchestration_tag = design.get_orchestration_tag(tag.clone());
        assert!(orchestration_tag.is_ok());
        assert_eq!(*orchestration_tag.unwrap().tag(), tag);
    }

    #[test]
    fn get_orchestration_tag_not_found() {
        let id = Tag::from_str_static("design1");
        let params = DesignConfig::default();
        let design = Design::new(id, params);

        let tag = Tag::from_str_static("non_existent_tag");

        // Attempt to retrieve a non-existent orchestration tag
        let orchestration_tag = design.get_orchestration_tag(tag);
        assert!(orchestration_tag.is_err());
    }

    // TODO add more tests once new Program skeleton is created
}
