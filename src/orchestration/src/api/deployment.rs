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

use std::rc::Rc;

use crate::{
    api::{
        design::{Design, DesignTag},
        OrchestrationApi,
    },
    common::tag::Tag,
    events::events_provider::ShutdownNotifier,
    program::ProgramBuilder,
};
use async_runtime::core::types::UniqueWorkerId;
use foundation::prelude::CommonErrors;

pub struct Deployment<'a, T> {
    api: &'a mut OrchestrationApi<T>,
}

impl<T> Deployment<'_, T> {
    pub fn new(api: &mut OrchestrationApi<T>) -> Deployment<'_, T> {
        Deployment { api }
    }

    /// Maps a system events to user events. This means that the specified user events will be treated as global events across all processes.
    pub fn bind_events_as_global(&mut self, system_event: &str, events_to_bind: &[Tag]) -> Result<(), CommonErrors> {
        let mut ret = Ok(());

        let creator = self.api.events.specify_global_event(system_event)?;

        for d in &mut self.api.designs {
            let _ = d.db.set_creator_for_events(Rc::clone(&creator), events_to_bind).or_else(|e| {
                ret = Err(e);
                ret
            });
        }

        ret
    }

    /// Binds user events to a local event. This means that the specified user events will be treated as local events within the process boundaries.
    pub fn bind_events_as_local(&mut self, events_to_bind: &[Tag]) -> Result<(), CommonErrors> {
        let mut ret = Ok(());

        let creator = self.api.events.specify_local_event()?;

        for d in &mut self.api.designs {
            let _ = d.db.set_creator_for_events(Rc::clone(&creator), events_to_bind).or_else(|e| {
                // TODO: This returns NotFound if a given event isn't in this particular design. Seems like an error?
                //       Not all events have to be on all designs.
                ret = Err(e);
                ret
            });
        }

        ret
    }

    /// Binds an invoke action to a worker across all designs wherever that invoke action is registered.
    /// The registered invoke action will always be executed by the specified worker.
    /// # Arguments
    /// * `tag` - The tag of the invoke action to bind.
    /// * `worker_id` - The unique identifier of the worker to bind the invoke action to.
    ///
    pub fn bind_invoke_to_worker(&mut self, tag: Tag, worker_id: UniqueWorkerId) -> Result<(), CommonErrors> {
        let mut ret = Ok(());

        for d in &mut self.api.designs {
            let _ = d.db.set_invoke_worker_id(tag, worker_id).or_else(|e| {
                ret = Err(e);
                ret
            });
        }

        ret
    }

    /// Binds a shutdown event as a global event.
    pub fn bind_shutdown_event_as_global(&mut self, system_event: &str, event: Tag) -> Result<(), CommonErrors> {
        let creator = self.api.events.specify_global_event(system_event)?;

        for design in &mut self.api.designs {
            let _ = design.db.set_creator_for_shutdown_event(Rc::clone(&creator), event);
        }

        Ok(())
    }

    /// Binds a shutdown event as a local event.
    pub fn bind_shutdown_event_as_local(&mut self, event: Tag) -> Result<(), CommonErrors> {
        let creator = self.api.events.specify_local_event()?;

        for design in &mut self.api.designs {
            let _ = design.db.set_creator_for_shutdown_event(Rc::clone(&creator), event);
        }

        Ok(())
    }

    /// Retrieve a shutdown notifier for the given event.
    pub fn get_shutdown_notifier(&self, event: Tag) -> Result<Box<dyn ShutdownNotifier>, CommonErrors> {
        // All designs share a creator for the same event, so return the first found.
        for design in &self.api.designs {
            if let Ok(creator) = design.db.get_creator_for_shutdown_event(event) {
                if let Some(shutdown_notifier) = creator.borrow_mut().create_shutdown_notifier() {
                    return Ok(shutdown_notifier);
                }
            }
        }

        Err(CommonErrors::NotFound)
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
