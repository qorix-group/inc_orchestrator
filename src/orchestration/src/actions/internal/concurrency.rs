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
use crate::actions::internal::action::ActionExecError;
use crate::common::tag::Tag;
#[cfg(feature = "runtime-api-mock")]
use async_runtime::testing::mock::*;

#[cfg(not(feature = "runtime-api-mock"))]
use async_runtime::*;

use async_runtime::futures::reusable_box_future::{ReusableBoxFuture, ReusableBoxFuturePool};
use async_runtime::futures::{FutureInternalReturn, FutureState};
use async_runtime::scheduler::join_handle::JoinHandle;
use foundation::containers::growable_vec::GrowableVec;
use foundation::containers::reusable_objects::ReusableObject;
use foundation::containers::reusable_vec_pool::ReusableVecPool;
use foundation::not_recoverable_error;
use iceoryx2_bb_container::vec::Vec;
use logging_tracing::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Builder for constructing a concurrency group of actions to be executed in parallel.
/// Allows adding multiple branches (actions) and finalizing into a [`Concurrency`] object.
pub struct ConcurrencyBuilder {
    actions: GrowableVec<Box<dyn ActionTrait>>,
}

/// Final concurrency object, ready for execution.
/// Holds the actions to be executed concurrently and manages their execution and result collection.
pub struct Concurrency {
    base: ActionBaseMeta,
    actions: Vec<Box<dyn ActionTrait>>,
    futures_vec_pool: ReusableVecPool<ActionMeta>,
}

impl ConcurrencyBuilder {
    /// Create a new concurrency builder.
    pub fn new() -> Self {
        Self {
            // Initialize with a growable vector for actions with a minimum capacity of 2.
            actions: GrowableVec::new(2),
        }
    }

    /// Add a new branch (concurrent action).
    /// Returns a mutable reference to self for chaining.
    pub fn with_branch(&mut self, action: Box<dyn ActionTrait>) -> &mut Self {
        self.actions.push(action);
        self
    }

    /// Finalize and return the concurrency object ready for execution.
    ///
    /// # Panics
    /// Panics if no branch is added.
    pub fn build(mut self) -> Box<Concurrency> {
        const REUSABLE_OBJECT_POOL_SIZE: usize = 1;
        self.actions.lock();
        let length = self.actions.len();
        assert!(length >= 1, "Concurrency requires at least one branch.");
        Box::new(Concurrency {
            base: ActionBaseMeta {
                tag: "orch::internal::concurrency".into(),
                reusable_future_pool: Concurrency::create_reusable_future_pool(REUSABLE_OBJECT_POOL_SIZE),
            },
            actions: self.actions.into(),
            futures_vec_pool: ReusableVecPool::<ActionMeta>::new(REUSABLE_OBJECT_POOL_SIZE, |_| Vec::new(length)),
        })
    }
}

impl Default for ConcurrencyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Concurrency {
    /// Internal async execution logic for concurrent actions.
    ///
    /// Spawns all actions as tasks, waits for all to complete, and returns the first error if any branch fails.
    async fn execute_impl(meta: Tag, mut futures_vec: ReusableObject<Vec<ActionMeta>>) -> ActionResult {
        for fut in futures_vec.iter_mut() {
            if let Some(future) = fut.take_future() {
                *fut = ActionMeta::Handle(safety::spawn_from_reusable(future));
            }
        }

        trace!(concurrent = ?meta, "Before joining branches");

        let joined = ConcurrencyJoin::new(futures_vec);
        let res = joined.await;

        trace!(concurrent = ?meta, ?res, "After joining branches");
        res
    }

    /// Creates a reusable future pool
    fn create_reusable_future_pool(pool_size: usize) -> ReusableBoxFuturePool<ActionResult> {
        let mut vec_pool = ReusableVecPool::<ActionMeta>::new(pool_size, |_| Vec::new(1));
        let vec = vec_pool.next_object().unwrap();
        ReusableBoxFuturePool::<ActionResult>::new(pool_size, Self::execute_impl("dummy".into(), vec))
    }
}

impl ActionTrait for Concurrency {
    /// Attempts to execute all branches concurrently, returning a reusable boxed future.
    fn try_execute(&mut self) -> ReusableBoxFutureResult {
        let mut futures_vec = self.futures_vec_pool.next_object()?;

        for action in self.actions.iter_mut() {
            // Each action is executed and its future is collected for concurrent execution.
            futures_vec.push(ActionMeta::new(action.try_execute()?));
        }

        self.base.reusable_future_pool.next(Self::execute_impl(self.base.tag, futures_vec))
    }

    fn name(&self) -> &'static str {
        "Concurrency"
    }

    fn dbg_fmt(&self, nest: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = " ".repeat(nest);
        writeln!(f, "{}|-{} - {:?}", indent, self.name(), self.base)?;
        self.actions.iter().try_for_each(|x| {
            writeln!(f, "{} |branch", indent)?;
            x.dbg_fmt(nest + 1, f)
        })
    }
}

/// Represents the state of an action in the concurrency group.
/// Can be empty, a future, or a running handle.
enum ActionMeta {
    Empty,
    Future(ReusableBoxFuture<ActionResult>),
    Handle(JoinHandle<ActionResult>),
}

impl ActionMeta {
    /// Wraps a future in an ActionMeta.
    fn new(fut: ReusableBoxFuture<ActionResult>) -> Self {
        ActionMeta::Future(fut)
    }

    /// Takes the future out of the ActionMeta, leaving it empty.
    fn take_future(&mut self) -> Option<ReusableBoxFuture<ActionResult>> {
        match std::mem::replace(self, ActionMeta::Empty) {
            ActionMeta::Future(fut) => Some(fut),
            other => {
                *self = other;
                None
            }
        }
    }

    /// Returns a mutable reference to the handle if present.
    fn handle(&mut self) -> Option<&mut JoinHandle<ActionResult>> {
        match self {
            ActionMeta::Handle(handle) => Some(handle),
            _ => None,
        }
    }
}

/// Future that waits for multiple [`JoinHandle`]s to complete.
/// Returns `Ready` once all are done, or cancels all remaining handles if any branch fails.
/// Uses FutureState to track polling state.
struct ConcurrencyJoin {
    handles: ReusableObject<Vec<ActionMeta>>,
    state: FutureState,
}

impl ConcurrencyJoin {
    /// Create a new `ConcurrencyJoin` for the given handles.
    fn new(handles: ReusableObject<Vec<ActionMeta>>) -> Self {
        Self {
            handles,
            state: FutureState::New,
        }
    }

    /// Handles polling all join handles, aborts on first error.
    /// Returns Ready if all are done or any branch fails, Pending otherwise.
    fn join_result(&mut self, cx: &mut Context<'_>) -> Poll<ActionResult> {
        let result: FutureInternalReturn<ActionResult> = match self.state {
            FutureState::New | FutureState::Polled => {
                // Poll all handles and collect results.
                let mut is_done = true;
                let mut action_execution_result = ActionResult::Ok(());

                for hnd in self.handles.iter_mut() {
                    if let Some(handle) = hnd.handle() {
                        let res = Pin::new(handle).poll(cx);
                        match res {
                            Poll::Ready(action_result) => {
                                *hnd = ActionMeta::Empty;
                                match action_result {
                                    Ok(result) if result.is_err() => {
                                        action_execution_result = result;
                                        break;
                                    }
                                    Err(_) => {
                                        action_execution_result = Err(ActionExecError::Internal);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            Poll::Pending => {
                                is_done = false;
                            }
                        }
                    }
                }

                // If any branch failed, abort the remaining handles.
                if action_execution_result.is_err() {
                    for hnd in self.handles.iter_mut() {
                        if let Some(handle) = hnd.handle() {
                            handle.abort();
                        }
                    }
                    FutureInternalReturn::ready(action_execution_result)
                } else if is_done {
                    // All handles are done and no errors, return Ok(())
                    FutureInternalReturn::ready(Ok(()))
                } else {
                    FutureInternalReturn::polled()
                }
            }
            FutureState::Finished => {
                not_recoverable_error!("Future polled after it finished!")
            }
        };
        self.state.assign_and_propagate(result)
    }
}

impl Future for ConcurrencyJoin {
    type Output = ActionResult;

    /// Polls the `ConcurrencyJoin` future.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Poll the join result and return the appropriate Poll state.
        self.join_result(cx)
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::testing::MockActionBuilder;
    use crate::testing::OrchTestingPoller;
    use std::task::Poll;
    use testing_macros::ensure_clear_mock_runtime;

    /// Test creating a concurrency object and adding branches.
    #[test]
    fn test_concurrency_builder() {
        let mock1 = MockActionBuilder::new().build();
        let mock2 = MockActionBuilder::new().build();
        // Create a concurrency builder using new() and add two branches.
        let mut concurrency_builder1 = ConcurrencyBuilder::new();
        concurrency_builder1.with_branch(Box::new(mock1)).with_branch(Box::new(mock2));
        let concurrency1 = concurrency_builder1.build();
        assert_eq!(concurrency1.actions.len(), 2);

        // Create a concurrency builder using default() and add a branch.
        let mock3 = MockActionBuilder::new().build();
        let mut concurrency_builder2 = ConcurrencyBuilder::default();
        concurrency_builder2.with_branch(Box::new(mock3));
        let concurrency2 = concurrency_builder2.build();
        assert_eq!(concurrency2.actions.len(), 1);

        assert_eq!(concurrency2.name(), "Concurrency");
    }

    /// Test that building without any branch panics.
    #[test]
    #[should_panic(expected = "Concurrency requires at least one branch.")]
    fn test_concurrency_builder_panics_with_no_branch() {
        let concurrency_builder = ConcurrencyBuilder::new();
        let _ = concurrency_builder.build();
    }

    /// Test concurrent execution where all actions succeed.
    #[test]
    #[ensure_clear_mock_runtime]
    fn test_concurrency_execute_ok_actions() {
        let mock1 = MockActionBuilder::new().will_once(Ok(())).build();
        let mock2 = MockActionBuilder::new().will_once(Ok(())).build();

        let mut concurrency_builder = ConcurrencyBuilder::new();
        concurrency_builder.with_branch(Box::new(mock1)).with_branch(Box::new(mock2));
        let mut concurrency = concurrency_builder.build();

        let mut poller = OrchTestingPoller::new(concurrency.try_execute().unwrap());
        // Call the poll function to spawn all the actions to execute concurrently and wait for them to complete.
        // The mock runtime will handle the execution of these actions.
        let mut result = poller.poll();

        // Start a thread to further call the poll function till,
        // - all the spawned actions are executed by the mock runtime
        // - Or abort if any action fails
        let handle = std::thread::spawn(move || {
            while result.is_pending() {
                result = poller.poll();
            }
            result
        });

        // Use the mock runtime to execute all spawned concurrent actions.
        let _x = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();

            assert_eq!(runtime.remaining_tasks(), 0);
        });

        // Get the result
        let result = handle.join().unwrap();
        assert!(matches!(result, Poll::Ready(Ok(()))));
    }

    /// Test concurrent execution where one action fails.
    #[test]
    #[ensure_clear_mock_runtime]
    fn test_concurrency_execute_err_action() {
        let mock1 = MockActionBuilder::new().will_once(Err(ActionExecError::NonRecoverableFailure)).build();

        let mut concurrency_builder = ConcurrencyBuilder::new();
        concurrency_builder.with_branch(Box::new(mock1));
        let mut concurrency = concurrency_builder.build();

        let mut poller = OrchTestingPoller::new(concurrency.try_execute().unwrap());
        // Call the poll function to spawn all the actions to execute concurrently and wait for them to complete.
        // The mock runtime will handle the execution of these actions.
        let mut result = poller.poll();

        // Start a thread to further call the poll function till,
        // - all the spawned actions are executed by the mock runtime
        // - Or abort if any action fails
        let handle = std::thread::spawn(move || {
            while result.is_pending() {
                result = poller.poll();
            }
            result
        });

        // Use the mock runtime to execute all spawned concurrent actions.
        let _x = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();

            assert_eq!(runtime.remaining_tasks(), 0);
        });

        // Get the result
        let result = handle.join().unwrap();
        assert!(matches!(result, Poll::Ready(Err(ActionExecError::NonRecoverableFailure))));
    }

    /// Test that pending actions are aborted if any branch fails.
    #[test]
    #[ensure_clear_mock_runtime]
    fn test_concurrency_abort_pending_actions() {
        let mock1 = MockActionBuilder::new().will_once(Ok(())).build();
        let mock2 = MockActionBuilder::new().will_once(Err(ActionExecError::Internal)).build();
        let mock3 = MockActionBuilder::new().will_once(Ok(())).build();
        let mock4 = MockActionBuilder::new().will_once(Ok(())).build();
        let mock5 = MockActionBuilder::new().will_once(Ok(())).build();

        let mut concurrency_builder = ConcurrencyBuilder::new();
        concurrency_builder
            .with_branch(Box::new(mock1))
            .with_branch(Box::new(mock2))
            .with_branch(Box::new(mock3))
            .with_branch(Box::new(mock4))
            .with_branch(Box::new(mock5));
        let mut concurrency = concurrency_builder.build();

        let mut poller = OrchTestingPoller::new(concurrency.try_execute().unwrap());
        // Call the poll function to spawn all the actions to execute concurrently and wait for them to complete.
        // The mock runtime will handle the execution of these actions.
        let mut result = poller.poll();

        // Start a thread to further call the poll function till,
        // - all the spawned actions are executed by the mock runtime
        // - Or abort if any action fails
        let handle = std::thread::spawn(move || {
            while result.is_pending() {
                result = poller.poll();
            }
            result
        });

        // Use the mock runtime to execute all spawned concurrent actions.
        let _x = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();

            assert_eq!(runtime.remaining_tasks(), 0);
        });

        // Get the result
        let result = handle.join().unwrap();
        assert!(matches!(result, Poll::Ready(Err(ActionExecError::Internal))));
    }

    /// Test that a concurrency object can be executed multiple times.
    #[test]
    #[ensure_clear_mock_runtime]
    fn test_concurrency_executed_twice() {
        let mock1 = MockActionBuilder::new().times(2).build();
        let mock2 = MockActionBuilder::new().times(2).build();

        let mut concurrency_builder = ConcurrencyBuilder::new();
        concurrency_builder.with_branch(Box::new(mock1)).with_branch(Box::new(mock2));
        let mut concurrency = concurrency_builder.build();

        // Execute the concurrency twice to ensure it can handle multiple executions correctly.
        // This is to test that the futures are reset and can be reused.
        for _ in 0..2 {
            let mut poller = OrchTestingPoller::new(concurrency.try_execute().unwrap());
            // Call the poll function to spawn all the actions to execute concurrently and wait for them to complete.
            // The mock runtime will handle the execution of these actions.
            let mut result = poller.poll();

            // Start a thread to further call the poll function till,
            // - all the spawned actions are executed by the mock runtime
            // - Or abort if any action fails
            let handle = std::thread::spawn(move || {
                while result.is_pending() {
                    result = poller.poll();
                }
                result
            });

            // Use the mock runtime to execute all spawned concurrent actions.
            let _x = async_runtime::testing::mock::runtime_instance(|runtime| {
                assert!(runtime.remaining_tasks() > 0);
                runtime.advance_tasks();

                assert_eq!(runtime.remaining_tasks(), 0);
            });

            // Get the result
            let result = handle.join().unwrap();
            assert!(matches!(result, Poll::Ready(Ok(()))));
        }
    }
}
