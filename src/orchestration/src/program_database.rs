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

use crate::actions::internal::{
    action::ActionTrait,
    invoke::{Invoke, InvokeFunctionType, InvokeResult},
};
use crate::common::orch_tag::{MapIdentifier, OrchestrationTag};
use crate::common::tag::Tag;
use crate::common::DesignConfig;
use async_runtime::core::types::UniqueWorkerId;
use foundation::prelude::*;
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use std::{
    boxed::Box,
    cell::RefCell,
    fmt::Debug,
    future::Future,
    rc::Rc,
    sync::{Arc, Mutex},
};

struct ActionData {
    tag: Tag,
    worker_id: Option<UniqueWorkerId>,
    generator: Box<dyn Fn(Tag, Option<UniqueWorkerId>) -> Box<dyn ActionTrait>>,
}

pub(crate) struct ActionProvider {
    clonable_invokes: SlotMap<ActionData>,
}

impl ActionProvider {
    pub(crate) fn new(clonable_invokes_capacity: usize) -> Self {
        Self {
            clonable_invokes: SlotMap::new(clonable_invokes_capacity),
        }
    }

    pub(crate) fn provide_invoke(&mut self, key: SlotMapKey) -> Option<Box<dyn ActionTrait>> {
        if let Some(data) = self.clonable_invokes.get(key) {
            Some((data.generator)(data.tag, data.worker_id))
        } else {
            None
        }
    }

    fn is_tag_unique(&self, tag: &Tag) -> bool {
        !self.clonable_invokes.iter().any(|(_, data)| data.tag == *tag)
    }
}

impl Debug for ActionProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActionProvider")
    }
}

pub struct ProgramDatabase {
    action_provider: Rc<RefCell<ActionProvider>>,
}

impl ProgramDatabase {
    /// Creates a new instance of `ProgramDatabase`.
    pub fn new(params: DesignConfig) -> Self {
        // TODO: Provider needs to keep DesignConfig probably so tags can have info from it
        Self {
            action_provider: Rc::new(RefCell::new(ActionProvider::new(params.db_params.clonable_invokes_capacity))),
        }
    }

    /// Registers a function as an invoke action that can be created multiple times.
    pub fn register_invoke_fn(&self, tag: Tag, action: InvokeFunctionType) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(ActionData {
                tag,
                worker_id: None,
                generator: Box::new(move |tag: Tag, worker_id: Option<UniqueWorkerId>| Invoke::from_fn(tag, action, worker_id)),
            }) {
                Ok(OrchestrationTag::new(
                    tag,
                    key,
                    MapIdentifier::ClonableInvokeMap,
                    Rc::clone(&self.action_provider),
                ))
            } else {
                Err(CommonErrors::NoSpaceLeft)
            }
        } else {
            Err(CommonErrors::AlreadyDone)
        }
    }

    /// Registers an async function as an invoke action that can be created multiple times.
    pub fn register_invoke_async<A, F>(&self, tag: Tag, action: A) -> Result<OrchestrationTag, CommonErrors>
    where
        A: Fn() -> F + 'static + Send + Clone,
        F: Future<Output = InvokeResult> + 'static + Send,
    {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(ActionData {
                tag,
                worker_id: None,
                generator: Box::new(move |tag: Tag, worker_id: Option<UniqueWorkerId>| Invoke::from_async(tag, action.clone(), worker_id)),
            }) {
                Ok(OrchestrationTag::new(
                    tag,
                    key,
                    MapIdentifier::ClonableInvokeMap,
                    Rc::clone(&self.action_provider),
                ))
            } else {
                Err(CommonErrors::NoSpaceLeft)
            }
        } else {
            Err(CommonErrors::AlreadyDone)
        }
    }

    /// Registers a method on an object as an invoke action.
    pub fn register_invoke_method<T: 'static + Send>(
        &self,
        tag: Tag,
        object: Arc<Mutex<T>>,
        method: fn(&mut T) -> InvokeResult,
    ) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(ActionData {
                tag,
                worker_id: None,
                generator: Box::new(move |tag: Tag, worker_id: Option<UniqueWorkerId>| {
                    Invoke::from_method(tag, Arc::clone(&object), method, worker_id)
                }),
            }) {
                Ok(OrchestrationTag::new(
                    tag,
                    key,
                    MapIdentifier::ClonableInvokeMap,
                    Rc::clone(&self.action_provider),
                ))
            } else {
                Err(CommonErrors::NoSpaceLeft)
            }
        } else {
            Err(CommonErrors::AlreadyDone)
        }
    }

    /// Registers an async method on an object as an invoke action.
    pub fn register_invoke_method_async<T, M, F>(&self, tag: Tag, object: Arc<Mutex<T>>, method: M) -> Result<OrchestrationTag, CommonErrors>
    where
        T: 'static + Send,
        M: Fn(Arc<Mutex<T>>) -> F + 'static + Send + Clone,
        F: Future<Output = InvokeResult> + 'static + Send,
    {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(ActionData {
                tag,
                worker_id: None,
                generator: Box::new(move |tag: Tag, worker_id: Option<UniqueWorkerId>| {
                    Invoke::from_method_async(tag, Arc::clone(&object), method.clone(), worker_id)
                }),
            }) {
                Ok(OrchestrationTag::new(
                    tag,
                    key,
                    MapIdentifier::ClonableInvokeMap,
                    Rc::clone(&self.action_provider),
                ))
            } else {
                Err(CommonErrors::NoSpaceLeft)
            }
        } else {
            Err(CommonErrors::AlreadyDone)
        }
    }

    /// Associates an invoke action with a tag with the given worker id.
    pub fn set_invoke_worker_id(&mut self, tag: Tag, worker_id: UniqueWorkerId) -> Result<(), CommonErrors> {
        let ap = &mut self.action_provider.borrow_mut();
        let map = &mut ap.clonable_invokes;

        if let Some((key, _)) = map.iter().find(|(_, data)| data.tag == tag) {
            // A mutable borrow is needed to take the data out of the entry, but iter_mut is not implemented for SlotMap.
            if let Some(data) = map.get_mut(key) {
                if data.worker_id.is_some() {
                    return Err(CommonErrors::AlreadyDone);
                }

                data.worker_id = Some(worker_id);

                return Ok(());
            }
        }

        Err(CommonErrors::NoData)
    }

    /// Returns an `OrchestrationTag` for an action previously registered with the given tag.
    pub fn get_orchestration_tag(&self, tag: Tag) -> Option<OrchestrationTag> {
        if let Some((key, entry)) = self.action_provider.borrow().clonable_invokes.iter().find(|(_, entry)| entry.tag == tag) {
            Some(OrchestrationTag::new(
                entry.tag,
                key,
                MapIdentifier::ClonableInvokeMap,
                Rc::clone(&self.action_provider),
            ))
        } else {
            None
        }
    }
}

impl Default for ProgramDatabase {
    fn default() -> Self {
        Self::new(DesignConfig::default())
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::{actions::internal::action::ActionExecError, testing::OrchTestingPoller};
    use std::task::Poll;
    use testing_macros::ensure_clear_mock_runtime;

    #[test]
    fn test_register_invoke_fn() {
        let pd = ProgramDatabase::default();

        fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        fn test2() -> InvokeResult {
            Err(0xbeef_u64.into())
        }

        let tag = pd.register_invoke_fn("tag1".into(), test1).unwrap();
        assert!(pd.register_invoke_fn("tag1".into(), test1).is_err());
        assert!(pd.register_invoke_fn("tag2".into(), test2).is_ok());

        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into()))));
    }

    #[test]
    fn test_register_invoke_async() {
        let pd = ProgramDatabase::default();

        async fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        async fn test2() -> InvokeResult {
            Err(0xbeef_u64.into())
        }

        let tag = pd.register_invoke_async("tag1".into(), test1).unwrap();
        assert!(pd.register_invoke_async("tag1".into(), test1).is_err());
        assert!(pd.register_invoke_async("tag2".into(), test2).is_ok());

        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into()))));
    }

    #[test]
    fn test_register_invoke_method() {
        let pd = ProgramDatabase::default();

        struct Test1 {}

        impl Test1 {
            fn test1(&mut self) -> InvokeResult {
                Err(0xcafe_u64.into())
            }
        }

        struct Test2 {}

        impl Test2 {
            fn test2(&mut self) -> InvokeResult {
                Err(0xbeef_u64.into())
            }
        }

        let obj1 = Arc::new(Mutex::new(Test1 {}));
        let obj2 = Arc::new(Mutex::new(Test2 {}));

        let tag = pd.register_invoke_method("tag1".into(), Arc::clone(&obj1), Test1::test1).unwrap();
        assert!(pd.register_invoke_method("tag1".into(), Arc::clone(&obj1), Test1::test1).is_err());
        assert!(pd.register_invoke_method("tag2".into(), Arc::clone(&obj2), Test2::test2).is_ok());

        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into()))));
    }

    #[test]
    fn test_register_invoke_method_async() {
        let pd = ProgramDatabase::default();

        struct Test1 {}

        async fn test1(object: Arc<Mutex<Test1>>) -> InvokeResult {
            let _guard = object.lock().unwrap();
            Err(0xcafe_u64.into())
        }

        struct Test2 {}

        async fn test2(object: Arc<Mutex<Test2>>) -> InvokeResult {
            let _guard = object.lock().unwrap();
            Err(0xbeef_u64.into())
        }

        let obj1 = Arc::new(Mutex::new(Test1 {}));
        let obj2 = Arc::new(Mutex::new(Test2 {}));

        let tag = pd.register_invoke_method_async("tag1".into(), Arc::clone(&obj1), test1).unwrap();
        assert!(pd.register_invoke_method_async("tag1".into(), Arc::clone(&obj1), test1).is_err());
        assert!(pd.register_invoke_method_async("tag2".into(), Arc::clone(&obj2), test2).is_ok());

        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));

        let tag = pd.get_orchestration_tag("tag2".into()).unwrap();
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xbeef_u64.into()))));
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_fn_with_worker_id() {
        let mut pd = ProgramDatabase::default();

        fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        let tag = pd.register_invoke_fn("tag1".into(), test1).unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        let _ = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();
            assert_eq!(runtime.remaining_tasks(), 0);
        });
        // Check the result.
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_async_with_worker_id() {
        let mut pd = ProgramDatabase::default();

        async fn test1() -> InvokeResult {
            Err(0xcafe_u64.into())
        }

        let tag = pd.register_invoke_async("tag1".into(), test1).unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        let _ = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();
            assert_eq!(runtime.remaining_tasks(), 0);
        });
        // Check the result.
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_method_with_worker_id() {
        let mut pd = ProgramDatabase::default();

        struct Test1 {}

        impl Test1 {
            fn test1(&mut self) -> InvokeResult {
                Err(0xcafe_u64.into())
            }
        }

        let tag = pd
            .register_invoke_method("tag1".into(), Arc::new(Mutex::new(Test1 {})), Test1::test1)
            .unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        let _ = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();
            assert_eq!(runtime.remaining_tasks(), 0);
        });
        // Check the result.
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));
    }

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_invoke_method_async_with_worker_id() {
        let mut pd = ProgramDatabase::default();

        struct Test1 {}

        async fn test1(object: Arc<Mutex<Test1>>) -> InvokeResult {
            let _guard = object.lock().unwrap();
            Err(0xcafe_u64.into())
        }

        let tag = pd
            .register_invoke_method_async("tag1".into(), Arc::new(Mutex::new(Test1 {})), test1)
            .unwrap();
        assert_eq!(pd.set_invoke_worker_id("tag1".into(), "worker_id".into()), Ok(()));
        let mut invoke = Invoke::from_tag(&tag);
        let mut poller = OrchTestingPoller::new(invoke.try_execute().unwrap());

        // Wait for invoke to schedule the action.
        let _ = poller.poll();
        // Run the action.
        let _ = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();
            assert_eq!(runtime.remaining_tasks(), 0);
        });
        // Check the result.
        assert_eq!(poller.poll(), Poll::Ready(Err(ActionExecError::UserError(0xcafe_u64.into()))));
    }
}
