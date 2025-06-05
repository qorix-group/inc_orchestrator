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

use std::ops::Deref;

use foundation::{containers::growable_vec::GrowableVec, prelude::CommonErrors};

use crate::{
    actions::internal::invoke,
    common::{orch_tag::OrchestrationTag, tag::Tag, DesignConfig},
    program::Program,
    program_database::ProgramDatabase,
};

pub type ProgramTag = Tag;

pub type DesignTag = Tag;

pub struct DesignConfigBounded(DesignConfig);

impl Deref for DesignConfigBounded {
    type Target = DesignConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Design {
    id: DesignTag,
    params: DesignConfig, // TODO: probably remove when we store it in ProgramDatabase
    db: ProgramDatabase,
    programs: GrowableVec<ProgramData>,
}

impl Design {
    pub fn new(id: DesignTag, params: DesignConfig) -> Self {
        const DEFAULT_PROGRAMS_CNT: usize = 1;
        Design {
            id,
            params,
            db: ProgramDatabase::new(params),
            programs: GrowableVec::new(DEFAULT_PROGRAMS_CNT),
        }
    }

    pub fn get_config(&self) -> DesignConfigBounded {
        DesignConfigBounded(self.params)
    }

    pub fn id(&self) -> Tag {
        self.id
    }

    pub fn register_invoke_fn(&self, tag: Tag, action: invoke::InvokeFunctionType) -> Result<OrchestrationTag, CommonErrors> {
        self.db.register_invoke_fn(tag, action)
    }

    pub fn get_orchestration_tag(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        self.db.get_orchestration_tag(tag).ok_or(CommonErrors::NotFound)
    }

    pub fn add_program<F>(&mut self, id: ProgramTag, program_creator: F)
    where
        F: FnOnce(&mut Self) -> Result<Program, CommonErrors> + 'static,
    {
        self.programs.push(ProgramData::new(id, Box::new(program_creator)));
    }

    pub fn has_any_programs(&self) -> bool {
        !self.programs.is_empty()
    }

    pub(super) fn get_programs(mut self, mut container: GrowableVec<Program>) -> Result<GrowableVec<Program>, CommonErrors> {
        while let Some(program_data) = self.programs.pop() {
            let program = (program_data.1)(&mut self)?;
            container.push(program);
        }

        Ok(container)
    }
}

type ProgramBuilderFn = Box<dyn FnOnce(&mut Design) -> Result<Program, CommonErrors>>;

#[allow(dead_code)]
pub(super) struct ProgramData(ProgramTag, ProgramBuilderFn);

impl ProgramData {
    pub(super) fn new(id: ProgramTag, program: ProgramBuilderFn) -> Self {
        Self(id, program)
    }
}

#[cfg(test)]
mod tests {

    use crate::actions::internal::action::UserErrValue;

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
