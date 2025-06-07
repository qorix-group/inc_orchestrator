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

use super::action::{ActionBaseMeta, ActionResult, ActionTrait, ReusableBoxFutureResult};
use crate::common::tag::Tag;

use async_runtime::futures::reusable_box_future::{ReusableBoxFuture, ReusableBoxFuturePool};
use foundation::{
    containers::{growable_vec::GrowableVec, reusable_objects::ReusableObject, reusable_vec_pool::ReusableVecPool},
    prelude::*,
};

const REUSABLE_FUTURE_POOL_SIZE: usize = 2;
const REUSABLE_VEC_POOL_SIZE: usize = 2;
const REUSABLE_VEC_SIZE: usize = 10;
const DEFAULT_TAG: &str = "orch::internal::sequence";

///
/// Construct a `SequenceBuilder` for creating a `Sequence` action
///
pub struct SequenceBuilder {
    actions: GrowableVec<Box<dyn ActionTrait>>,
}

impl Default for SequenceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceBuilder {
    ///
    /// Construct a `SequenceBuilder`
    ///
    pub fn new() -> SequenceBuilder {
        Self {
            actions: GrowableVec::new(REUSABLE_VEC_SIZE),
        }
    }

    ///
    /// Add an action to the `Sequence`
    ///
    pub fn with_step(&mut self, action: Box<dyn ActionTrait>) -> &mut Self {
        self.actions.push(action);
        self
    }

    ///
    /// Build the `Sequence` action
    ///
    /// # Errors
    ///
    /// Returns ``Err(NoData)` if no reusable futures collection for storing the actions' futures is available
    ///
    /// # Panics
    ///
    /// Panics if the `Sequence` does not contain any actions
    ///
    pub fn build(&mut self) -> Result<Box<Sequence>, CommonErrors> {
        assert!(!self.actions.is_empty(), "Sequence must contain at least one action!");

        // No more actions may be added beyond this point
        self.actions.lock();

        // Create a pool of reusable futures' collection for storing the actions' futures_pool
        let mut futures_vec_pool = ReusableVecPool::<ReusableBoxFuture<ActionResult>>::new(REUSABLE_VEC_POOL_SIZE, |_| Vec::new(self.actions.len()));

        // Populate the futures' collection to initialize the reusable future pool's layout
        let reusable_future_pool = ReusableBoxFuturePool::<ActionResult>::new(
            REUSABLE_FUTURE_POOL_SIZE,
            Sequence::execute_impl(Tag::from_str_static(DEFAULT_TAG), futures_vec_pool.next_object().unwrap()),
        );

        // Move the actions from Builder's GrowableVec to Sequence's fixed-sized Vec
        let mut actions = Vec::<Box<dyn ActionTrait>>::new(self.actions.len());
        while let Some(action) = self.actions.pop() {
            actions.push(action);
        }

        // Finally, return the `Sequence` action
        Ok(Box::new(Sequence {
            actions,
            base: ActionBaseMeta {
                tag: Tag::from_str_static(DEFAULT_TAG),
                reusable_future_pool,
            },
            futures_vec_pool,
        }))
    }
}

///
/// An orchestration action that invokes subsequent actions specified via `with_step()` in a FIFO
/// manner.
///
/// If any action encounters an error, the `Sequence` execution will terminate immediately,
/// preventing the execution of any remaining actions.
///
pub struct Sequence {
    actions: Vec<Box<dyn ActionTrait>>,
    base: ActionBaseMeta,
    futures_vec_pool: ReusableVecPool<ReusableBoxFuture<ActionResult>>,
}

impl Sequence {
    async fn execute_impl(tag: Tag, mut futures: ReusableObject<Vec<ReusableBoxFuture<ActionResult>>>) -> ActionResult {
        trace!(sequence = ?tag, "Before awaiting steps");

        // Execute all futures in the collection, but terminates immediately upon error
        // We can directly pop() without reversing the order here, because the reversion already took place
        // during elements transfer from Builder's GrowableVec to Sequence's Vec
        while let Some(future) = futures.pop() {
            trace!(step = ?tag, "Before awaiting step");
            let result = future.into_pin().await;
            if result.is_err() {
                // Terminate sequence and propagate the error
                error!("Error in sequence step {:?}", tag);
                return result;
            }
            trace!(step = ?tag, "After awaiting step");
        }

        trace!(sequence = ?tag, "After awaiting steps",);
        Ok(())
    }
}

impl ActionTrait for Sequence {
    fn try_execute(&mut self) -> ReusableBoxFutureResult {
        // Get a fresh reusable futures collection and re-populate it with actions' futures
        let mut futures_vec_pool = self.futures_vec_pool.next_object().unwrap();
        self.actions.iter_mut().try_for_each(|action| {
            // Return error in (unlikely) case that no more future can be added to the reusable collection
            if !futures_vec_pool.push(action.try_execute()?) {
                error!("Unable to add a future to the reusable future vec in {:?}", self.base);
                return Err(CommonErrors::NoSpaceLeft);
            }
            Ok(())
        })?;

        // Get a future from the reusable future pool and execute it
        self.base
            .reusable_future_pool
            .next(Sequence::execute_impl(self.base.tag, futures_vec_pool))
    }

    fn name(&self) -> &'static str {
        "Sequence"
    }

    fn dbg_fmt(&self, nest: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = " ".repeat(nest);
        writeln!(f, "{}|-{} - {:?}", indent, self.name(), self.base)?;
        self.actions.iter().try_for_each(|action| {
            writeln!(f, "{} |step", indent)?;
            action.dbg_fmt(nest + 1, f)
        })
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::actions::internal::action::{ActionExecError, UserErrValue};
    use crate::testing::{MockActionBuilder, OrchTestingPoller};

    use std::task::Poll;

    #[test]
    #[should_panic]
    fn test_build_on_empty_sequence_should_panic() {
        let mut seq = SequenceBuilder::new().build().unwrap();

        let mut mock = OrchTestingPoller::new(seq.try_execute().unwrap());
        assert_eq!(Poll::Ready(Ok(())), mock.poll());
    }

    #[test]
    fn test_all_subsequent_steps_are_called() {
        let mock_1 = Box::new(MockActionBuilder::new().times(1).build());
        let mock_2 = Box::new(MockActionBuilder::new().times(1).build());
        let mut seq = SequenceBuilder::new().with_step(mock_1).with_step(mock_2).build().unwrap();

        let mut mock = OrchTestingPoller::new(seq.try_execute().unwrap());
        assert_eq!(Poll::Ready(Ok(())), mock.poll());
    }

    #[test]
    fn test_all_nested_steps_are_called() {
        let mock_1 = Box::new(MockActionBuilder::new().times(1).build());
        let mock_nested_a = Box::new(MockActionBuilder::new().times(1).build());
        let mock_nested_b = Box::new(MockActionBuilder::new().times(1).build());
        let mock_2 = Box::new(MockActionBuilder::new().times(1).build());
        let mut seq = SequenceBuilder::new()
            .with_step(mock_1)
            .with_step(SequenceBuilder::new().with_step(mock_nested_a).with_step(mock_nested_b).build().unwrap())
            .with_step(mock_2)
            .build()
            .unwrap();

        let mut mock = OrchTestingPoller::new(seq.try_execute().unwrap());
        assert_eq!(Poll::Ready(Ok(())), mock.poll());
    }

    #[test]
    fn test_step_with_err_terminates_immediately() {
        let mock_ok = Box::new(MockActionBuilder::new().will_once(Ok(())).build());
        let user_err = ActionExecError::UserError(UserErrValue::from(42));
        let mock_err_1 = Box::new(MockActionBuilder::new().will_once(Err(user_err)).build());
        let mock_err_2 = Box::new(MockActionBuilder::new().times(0).build());
        let mut seq = SequenceBuilder::new()
            .with_step(mock_ok)
            .with_step(mock_err_1)
            .with_step(mock_err_2)
            .build()
            .unwrap();

        let mut mock = OrchTestingPoller::new(seq.try_execute().unwrap());
        assert_eq!(Poll::Ready(Err(user_err)), mock.poll());
    }
}
