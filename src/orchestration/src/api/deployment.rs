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

use foundation::prelude::CommonErrors;

use crate::{
    api::{
        design::{Design, DesignTag},
        OrchestrationApi,
    },
    common::tag::Tag,
    program::ProgramBuilder,
};

pub struct Deployment<'a, T> {
    api: &'a mut OrchestrationApi<T>,
}

impl<T> Deployment<'_, T> {
    pub fn new(api: &mut OrchestrationApi<T>) -> Deployment<'_, T> {
        Deployment { api }
    }

    /// Maps a system events to user events. This means that the specified user events will be treated as global events across all processes.
    pub fn bind_events_as_global(&mut self, system_event: &str, user_events_to_bind: &[Tag]) -> Result<(), CommonErrors> {
        let mut ret = Ok(());

        let creator = self.api.events.specify_global_event(system_event)?;

        for d in &mut self.api.designs {
            let _ = d.db.set_event_type(creator.clone(), user_events_to_bind).or_else(|e| {
                ret = Err(e);
                ret
            });
        }

        ret
    }

    /// Binds user events to a local event. This means that the specified user events will be treated as local events within the process boundaries.
    pub fn bind_events_as_local(&mut self, user_events_to_bind: &[Tag]) -> Result<(), CommonErrors> {
        let mut ret = Ok(());

        let creator = self.api.events.specify_local_event()?;

        for d in &mut self.api.designs {
            let _ = d.db.set_event_type(creator.clone(), user_events_to_bind).or_else(|e| {
                ret = Err(e);
                ret
            });
        }

        ret
    }

    /// Adds a program to the design. The program is created using the provided closure, which receives a mutable reference to the design.
    ///
    /// # Returns
    /// `Ok(())` if the program was added successfully
    /// `Err(CommonErrors::AlreadyDone)` if the design already has programs
    /// `Err(CommonErrors::NotFound)` if the design with the specified tag was not
    ///
    pub fn add_program<F>(&mut self, design_tag: DesignTag, program: F, name: &'static str) -> Result<(), CommonErrors>
    where
        F: FnOnce(&mut Design, &mut ProgramBuilder) -> Result<(), CommonErrors> + 'static,
    {
        let p = &mut self.api.designs.iter_mut().find(|d| d.id() == design_tag);

        if let Some(design) = p {
            if design.has_any_programs() {
                Err(CommonErrors::AlreadyDone)
            } else {
                design.add_program(name, Box::new(program));
                Ok(())
            }
        } else {
            Err(CommonErrors::NotFound)
        }
    }
}
