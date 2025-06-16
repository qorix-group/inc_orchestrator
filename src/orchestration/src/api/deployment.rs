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
        design::{Design, DesignTag, ProgramTag},
        OrchestrationApi,
    },
    program::internal::Program,
};

pub struct Deployment<'a, T> {
    api: &'a mut OrchestrationApi<T>,
}

impl<T> Deployment<'_, T> {
    pub fn new(api: &mut OrchestrationApi<T>) -> Deployment<'_, T> {
        Deployment { api }
    }

    pub fn add_program<F>(&mut self, design_tag: DesignTag, program: F, tag: ProgramTag) -> Result<(), CommonErrors>
    where
        F: FnOnce(&mut Design) -> Result<Program, CommonErrors> + 'static,
    {
        let p = &mut self.api.designs.iter_mut().find(|d| d.id() == design_tag);

        if let Some(design) = p {
            if design.has_any_programs() {
                Err(CommonErrors::AlreadyDone)
            } else {
                design.add_program(tag, Box::new(program));
                Ok(())
            }
        } else {
            Err(CommonErrors::NotFound)
        }
    }
}

// TODO add more tests once new Program skeleton is created
