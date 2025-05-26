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

use crate::actions::internal::action::{ActionResult, ActionTrait, ReusableBoxFutureResult};

use async_runtime::futures::reusable_box_future::ReusableBoxFuturePool;
use testing::mock_fn::{CallableTrait, MockFn, MockFnBuilder};

const DEFAULT_POOL_SIZE: usize = 5;

///
/// Helper mock object
///
pub struct MockActionBuilder(MockFnBuilder<ActionResult>);

pub struct MockAction {
    mockfn: MockFn<ActionResult>,
    pool: ReusableBoxFuturePool<ActionResult>,
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
        self.0 = self.0.clone().times(count);
        self
    }

    ///
    /// Ensure that the call() is invoked at least one more time and the call() returns the ret_val
    ///
    pub fn will_once(mut self, ret_val: ActionResult) -> Self {
        self.0 = self.0.clone().will_once(ret_val);
        self
    }

    ///
    /// Allow the call() to be invoked multiple times and the call() returns the ret_val
    /// If used, will_repeatedly() must be called the last
    ///
    pub fn will_repeatedly(mut self, ret_val: ActionResult) -> Self {
        self.0 = self.0.clone().will_repeatedly(ret_val);
        self
    }

    pub fn build(self) -> MockAction {
        let task = self.0.clone().build().call();
        MockAction {
            mockfn: self.0.build(),
            pool: ReusableBoxFuturePool::<ActionResult>::new(DEFAULT_POOL_SIZE, async move { task }),
        }
    }
}

impl ActionTrait for MockAction {
    fn try_execute(&mut self) -> ReusableBoxFutureResult {
        let result = self.mockfn.call();
        self.pool.next(async { result })
    }

    fn name(&self) -> &'static str {
        "MockAction"
    }

    fn dbg_fmt(&self, nest: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = " ".repeat(nest);
        writeln!(f, "{}|-{}", indent, self.name())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use foundation::prelude::CommonErrors;
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
        let mut mock = MockActionBuilder::new().will_once(Err(CommonErrors::Timeout)).build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::Timeout)));
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
        let mut mock = MockActionBuilder::new().will_repeatedly(Err(CommonErrors::GenericError)).build();

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::GenericError)));
        }
    }

    #[test]
    fn test_with_calls_equals_times_ok() {
        let mut mock = MockActionBuilder::new().times(3).will_repeatedly(Err(CommonErrors::Panicked)).build();

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::Panicked)));
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
            .will_once(Err(CommonErrors::AlreadyDone))
            .will_once(Err(CommonErrors::GenericError))
            .build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::AlreadyDone)));

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::GenericError)));
    }

    #[test]
    fn test_with_all_clauses() {
        let mut mock = MockActionBuilder::new()
            .times(5)
            .will_once(Ok(()))
            .will_once(Err(CommonErrors::OperationAborted))
            .will_repeatedly(Err(CommonErrors::AlreadyDone))
            .build();

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Ok(())));

        let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
        assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::OperationAborted)));

        for _ in 0..3 {
            let mut poller = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
            assert_eq!(poller.poll(), Poll::Ready(Err(CommonErrors::AlreadyDone)));
        }
    }

    #[test]
    #[should_panic]
    fn test_with_clause_after_repeated_should_panic() {
        let mut mock = MockActionBuilder::new()
            .will_repeatedly(Err(CommonErrors::AlreadyDone))
            .will_once(Err(CommonErrors::OperationAborted))
            .build();

        let _ = TestingFuturePoller::<ActionResult>::new(mock.try_execute().unwrap().into_pin());
    }
}
