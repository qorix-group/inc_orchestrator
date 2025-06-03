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

#![allow(dead_code)]

use std::task::{Poll, Waker};

use crate::actions::internal::action::{ActionResult, ActionTrait, ReusableBoxFutureResult};

use async_runtime::futures::reusable_box_future::{ReusableBoxFuture, ReusableBoxFuturePool};
use testing::{
    mock_fn::{CallableTrait, MockFn, MockFnBuilder},
    poller::TestingFuturePoller,
};

const DEFAULT_POOL_SIZE: usize = 5;

///
/// A mock object that can be used to monitor the invocation count of actions, i.e. try_execute().
/// Each invocation returns a (reusable) future containing values previously configured via will_once() or will_repeatedly().
///
pub struct MockActionBuilder(MockFnBuilder<ActionResult>);

pub struct MockAction {
    mockfn: MockFn<ActionResult>,
    pool: ReusableBoxFuturePool<ActionResult>,
}

impl Default for MockAction {
    fn default() -> Self {
        MockActionBuilder::default().build()
    }
}

impl Default for MockActionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MockActionBuilder {
    pub fn new() -> MockActionBuilder {
        Self(MockFnBuilder::<ActionResult>::new_with_default(Ok(())))
    }

    ///
    /// Set how many times exactly the try_execute() must be invoked
    ///
    pub fn times(mut self, count: usize) -> Self {
        self.0 = self.0.times(count);
        self
    }

    ///
    /// Ensure that the try_execute() is invoked at least one more time and the try_execute() returns the ret_val
    ///
    pub fn will_once(mut self, ret_val: ActionResult) -> Self {
        self.0 = self.0.will_once(ret_val);
        self
    }

    ///
    /// Allow the try_execute() to be invoked multiple times and the invokation returns the ret_val
    /// If used, will_repeatedly() must be called the last
    ///
    pub fn will_repeatedly(mut self, ret_val: ActionResult) -> Self {
        self.0 = self.0.will_repeatedly(ret_val);
        self
    }

    ///
    /// Create the MockAction instance based on the current configuration and initialize the reusable future pool
    ///
    pub fn build(self) -> MockAction {
        // The pool needs to know the future layout in advance, so we create a future "template" using another MockFn with identical OutType.
        // We can not re-use the one from self.0 for this purpose, since invoking build() results in the transient MockFn being inspected for
        // its call counts at drop(), leading to panics even before polling
        let dummy_task = MockFnBuilder::<ActionResult>::new_with_default(Ok(())).build().call();
        MockAction {
            mockfn: self.0.build(),
            pool: ReusableBoxFuturePool::<ActionResult>::new(DEFAULT_POOL_SIZE, async move { dummy_task }),
        }
    }
}

impl ActionTrait for MockAction {
    ///
    /// Return a "fresh" future that returns the current MockFn's call() result
    ///
    fn try_execute(&mut self) -> ReusableBoxFutureResult {
        let result = self.mockfn.call();
        self.pool.next(async move { result })
    }

    fn name(&self) -> &'static str {
        "MockAction"
    }

    fn dbg_fmt(&self, nest: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = " ".repeat(nest);
        writeln!(f, "{}|-{}", indent, self.name())
    }
}

pub struct OrchTestingPoller {
    poller: TestingFuturePoller<ActionResult>,
    waker: Waker,
}

impl OrchTestingPoller {
    pub fn new(future: ReusableBoxFuture<ActionResult>) -> Self {
        Self {
            poller: TestingFuturePoller::new(future.into_pin()),
            waker: async_runtime::testing::get_task_based_waker(),
        }
    }

    pub fn poll(&mut self) -> Poll<ActionResult> {
        self.poller.poll_with_waker(&self.waker)
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {

    use crate::actions::internal::action::ActionExecError;

    use super::*;

    use testing::poller::TestingFuturePoller;

    use std::task::Poll;

    #[test]
    fn test_once_ok() {
        let mut mock = MockActionBuilder::new().will_once(Ok(())).build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Ok(())));
    }

    #[test]
    fn test_once_err() {
        let mut mock = MockActionBuilder::new().will_once(Err(ActionExecError::Internal)).build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::Internal)));
    }

    #[test]
    fn test_repeatedly_ok() {
        let mut mock = MockActionBuilder::new().will_repeatedly(Ok(())).build();

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Ok(())));
        }
    }

    #[test]
    fn test_repeatedly_err() {
        let mut mock = MockActionBuilder::new()
            .will_repeatedly(Err(ActionExecError::NonRecoverableFailure))
            .build();

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::NonRecoverableFailure)));
        }
    }

    #[test]
    fn test_with_calls_equals_times_ok() {
        let mut mock = MockActionBuilder::new().times(3).will_repeatedly(Err(ActionExecError::Internal)).build();

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::Internal)));
        }
    }

    #[test]
    #[should_panic]
    fn test_with_less_calls_than_times_should_panic() {
        let mut mock = MockActionBuilder::new().times(3).build();

        for _ in 0..2 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Ok(())));
        }
    }

    #[test]
    #[should_panic]
    fn test_with_more_counts_than_times_should_panic() {
        let mut mock = MockActionBuilder::new().times(3).build();

        for _ in 0..4 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Ok(())));
        }
    }

    #[test]
    fn test_with_multiple_once_err() {
        let mut mock = MockActionBuilder::new()
            .will_once(Err(ActionExecError::Internal))
            .will_once(Err(ActionExecError::NonRecoverableFailure))
            .build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::Internal)));

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::NonRecoverableFailure)));
    }

    #[test]
    fn test_with_all_clauses() {
        let mut mock = MockActionBuilder::new()
            .times(5)
            .will_once(Ok(()))
            .will_once(Err(ActionExecError::NonRecoverableFailure))
            .will_repeatedly(Err(ActionExecError::Internal))
            .build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Ok(())));

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::NonRecoverableFailure)));

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::Internal)));
        }
    }

    #[test]
    #[should_panic]
    fn test_with_clause_after_repeated_should_panic() {
        let mut mock = MockActionBuilder::new()
            .will_repeatedly(Err(ActionExecError::Internal))
            .will_once(Err(ActionExecError::NonRecoverableFailure))
            .build();

        let _ = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
    }
}
