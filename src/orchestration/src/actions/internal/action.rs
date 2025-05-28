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

use crate::common::tag::Tag;

use async_runtime::futures::reusable_box_future::ReusableBoxFuture;
use foundation::prelude::CommonErrors;

use std::{
    fmt::{Debug, Formatter},
    ops::Deref,
};

/// Represents a user-defined error value that can be propagated through the action execution chain.
/// This allows user code to signal specific errors that can be handled or logged by the orchestrator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserErrValue(u64);

impl Deref for UserErrValue {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for UserErrValue {
    fn from(value: u64) -> Self {
        UserErrValue(value)
    }
}

#[allow(clippy::from_over_into)]
impl Into<ActionExecError> for UserErrValue {
    fn into(self) -> ActionExecError {
        ActionExecError::UserError(self)
    }
}

/// Enum representing possible errors that can occur during action execution.
///
/// Variants:
/// - `UserError(UserErrValue)`: Indicates an error returned by user code, allowing it to propagate through the chain. It means signature to `Invoke` needs to capture Futures/functions with Result<(), UserErrValue>
/// - `NonRecoverableFailure`: Represents a failure that cannot be recovered from.
/// - `Internal`: Placeholder for internal errors, with potential for expansion as needed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActionExecError {
    UserError(UserErrValue),
    NonRecoverableFailure,
    Internal, // TODO add more errors if needed
}

///
/// Result to indicate the given action status. [`Ok(())`] if everything went fine, Err(ActionExecError) to mark error in execution.
///
pub type ActionResult = Result<(), ActionExecError>;

///
/// Result to indicate the acquisition status of the reusable (boxed) future. [`Ok(ReusableBoxFuture<ActionResult>)`] if everything went fine, Err(CommonErrors) to mark error in execution.
///
pub type ReusableBoxFutureResult = Result<ReusableBoxFuture<ActionResult>, CommonErrors>;

///
/// Describes action interface that let us build task chain from program.
///
pub trait ActionTrait: Send {
    ///
    /// Will be called on each `Program` iteration.
    ///
    /// Key assumptions:
    ///     - should avoid allocation due to the usage of reusable boxed future
    ///     - each action shall propagate ActionResult down the chain in Future and should immediately stop it's work once Err(_) is reached, propagating it down.
    ///
    fn try_execute(&mut self) -> ReusableBoxFutureResult;

    ///
    /// Provide name of the action
    ///
    fn name(&self) -> &'static str;

    ///
    /// Since we store actions behind dyn ActionTrait, we need an API that we can call from program to print constructed representation
    ///
    fn dbg_fmt(&self, nest: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

#[derive(Clone, Copy)]
pub struct ActionBaseMeta {
    pub tag: Tag,
}

impl Debug for ActionBaseMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.tag)
    }
}
