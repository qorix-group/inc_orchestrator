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

// Additional remarks:
//
// - Actions shall follow API aka build pattern (more or less like in Nico code) for construction
// - First we need tree actions: Sequence, Concurrent and Invoke
// - Invoke shall be able to take from user - a function, an async function and object + method for a moment.
//

use crate::actions::action::*;
use logging_tracing::prelude::*;
use std::{
    fmt::Debug,
    time::{Duration, Instant},
};

///
/// Whole description to Task Chain is delivered via this instance. It shall hold all actions that build as Task Chain
///
pub struct Program {
    name: String,
    action: Option<Box<dyn ActionTrait>>,
}

impl Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program - {}", self.name)?;
        writeln!(f, "Body:")?;
        self.action.as_ref().unwrap().dbg_fmt(1, f)
    }
}

pub struct ProgramBuilder(Program);

impl ProgramBuilder {
    pub fn new(name: &str) -> Self {
        Self(Program {
            name: name.to_string(),
            action: None,
        })
    }

    pub fn with_body(&mut self, action: Box<dyn ActionTrait>) -> &mut Self {
        self.0.action = Some(action);
        self
    }

    pub fn build(mut self) -> Program {
        self.0.action.as_mut().expect("Body must be set for program!");
        self.0
    }
}

impl Program {
    ///
    /// Shall start running a task chain in a `loop`. This means that once TaskChain finishes, it will start from beginning until requested to stop.
    ///
    pub async fn run(&mut self) -> ActionResult {
        self.internal_run(None).await
    }

    ///
    /// Shall start running a task chain `N` times
    ///
    pub async fn run_n(&mut self, n: usize) -> ActionResult {
        self.internal_run(Some(n)).await
    }

    ///
    /// Should notify program to stop executing as soon as possible.
    ///
    pub fn stop() {
        todo!()
    }

    async fn internal_run(&mut self, n: Option<usize>) -> ActionResult {
        let iteration_count: usize = n.unwrap_or_default();
        let mut iteration = 0_usize;

        while n.is_none() || iteration < iteration_count {
            let start_time = Instant::now();

            let stats = ProgramStats {
                name: self.name.as_str(),
                iteration,
                iteration_time: Duration::ZERO,
            };
            trace!(meta = ?stats, "Iteration started");

            let future = self.action.as_mut().unwrap().try_execute();
            if future.is_err() {
                trace!("Failed to execute action");
                return Err(ActionExecError::Internal);
            }

            let result = async_runtime::spawn_from_reusable(future.unwrap()).await;

            if result.is_err() {
                trace!("Failed to execute action");
                return Err(ActionExecError::Internal);
            }

            result.unwrap()?;

            let stats = ProgramStats {
                name: self.name.as_str(),
                iteration,
                iteration_time: Instant::now().duration_since(start_time),
            };
            trace!( meta = ?stats, "Iteration completed");

            iteration += 1;
        }

        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct ProgramStats<'a> {
    name: &'a str,
    iteration: usize,
    iteration_time: Duration,
}
