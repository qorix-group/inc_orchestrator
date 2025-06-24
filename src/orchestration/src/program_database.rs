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

use crate::common::orch_tag::{MapIdentifier, OrchestrationTag};
use crate::common::tag::{AsTagTrait, Tag};
use crate::common::DesignConfig;
use crate::events::events_provider::{DesignEvent, EventActionType, DEFAULT_EVENTS_CAPACITY};
use crate::{
    actions::{
        action::ActionTrait,
        invoke::{Invoke, InvokeFunctionType, InvokeResult},
    },
    events::events_provider::EventCreator,
};
use async_runtime::core::types::UniqueWorkerId;
use foundation::containers::growable_vec::GrowableVec;
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
    design_events: SlotMap<DesignEvent>,
}

impl ActionProvider {
    pub(crate) fn new(clonable_invokes_capacity: usize) -> Self {
        Self {
            clonable_invokes: SlotMap::new(clonable_invokes_capacity),
            design_events: SlotMap::new(DEFAULT_EVENTS_CAPACITY),
        }
    }

    pub(crate) fn provide_invoke(&mut self, key: SlotMapKey) -> Option<Box<dyn ActionTrait>> {
        if let Some(data) = self.clonable_invokes.get(key) {
            Some((data.generator)(data.tag, data.worker_id))
        } else {
            None
        }
    }

    pub(crate) fn provide_event(&mut self, key: SlotMapKey, t: EventActionType) -> Option<Box<dyn ActionTrait>> {
        match t {
            EventActionType::Trigger => self.design_events.get(key).and_then(|e| e.creator()?.borrow_mut().create_trigger()),
            EventActionType::Sync => self.design_events.get(key).and_then(|e| e.creator()?.borrow_mut().create_sync()),
        }
    }
}

impl Debug for ActionProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActionProvider")
    }
}

pub struct ProgramDatabase {
    action_provider: Rc<RefCell<ActionProvider>>,
    design_shutdown_events: GrowableVec<DesignEvent>,
}

impl ProgramDatabase {
    /// Creates a new instance of `ProgramDatabase`.
    pub fn new(params: DesignConfig) -> Self {
        // TODO: Provider needs to keep DesignConfig probably so tags can have info from it
        Self {
            action_provider: Rc::new(RefCell::new(ActionProvider::new(params.db_params.clonable_invokes_capacity))),
            design_shutdown_events: GrowableVec::default(),
        }
    }

    pub fn register_event(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        if tag.is_in_collection(ap.design_events.iter()) || tag.is_in_collection(self.design_shutdown_events.iter()) {
            return Err(CommonErrors::AlreadyDone);
        }

        ap.design_events
            .insert(DesignEvent::new(tag))
            .ok_or(CommonErrors::NoSpaceLeft)
            .map(|key| OrchestrationTag::new(tag, key, MapIdentifier::Event, Rc::clone(&self.action_provider)))
    }

    pub fn register_shutdown_event(&mut self, tag: Tag) -> Result<(), CommonErrors> {
        let ap = self.action_provider.borrow_mut();

        if tag.is_in_collection(ap.design_events.iter()) || tag.is_in_collection(self.design_shutdown_events.iter()) {
            return Err(CommonErrors::AlreadyDone);
        }

        if self.design_shutdown_events.push(DesignEvent::new(tag)) {
            Ok(())
        } else {
            Err(CommonErrors::GenericError)
        }
    }

    /// Registers a function as an invoke action that can be created multiple times.
    pub fn register_invoke_fn(&self, tag: Tag, action: InvokeFunctionType) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        if !tag.is_in_collection(ap.clonable_invokes.iter()) {
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

        if !tag.is_in_collection(ap.clonable_invokes.iter()) {
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

        if !tag.is_in_collection(ap.clonable_invokes.iter()) {
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

        if !tag.is_in_collection(ap.clonable_invokes.iter()) {
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

        if let Some((key, _)) = tag.find_in_collection(map.iter()) {
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
    ///
    /// # Returns
    /// - `Ok(OrchestrationTag)` if the tag exists and is associated with an action.
    /// - `Err(CommonErrors::NotFound)` if the tag does not exist.
    /// - `Err(CommonErrors::GenericError)` if the tag is associated ambiguously (since we allow same tag for invoke/events/others)
    ///
    pub fn get_orchestration_tag(&self, tag: Tag) -> Result<OrchestrationTag, CommonErrors> {
        let ap = self.action_provider.borrow();

        let invoke = tag.find_in_collection(ap.clonable_invokes.iter());
        let evt = tag.find_in_collection(ap.design_events.iter()).map(|(key, _)| key);

        if evt.is_some() && invoke.is_some() {
            return Err(CommonErrors::GenericError);
        }

        if let Some((key, entry)) = invoke {
            Ok(OrchestrationTag::new(
                entry.tag,
                key,
                MapIdentifier::ClonableInvokeMap,
                Rc::clone(&self.action_provider),
            ))
        } else if let Some(key) = evt {
            Ok(OrchestrationTag::new(tag, key, MapIdentifier::Event, Rc::clone(&self.action_provider)))
        } else {
            Err(CommonErrors::NotFound)
        }
    }

    pub(crate) fn set_creator_for_events(&self, creator: EventCreator, user_events: &[Tag]) -> Result<(), CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();
        let mut ret = Ok(());

        for event in user_events {
            let item = event.find_in_collection(ap.design_events.iter());

            if let Some((key, _)) = item {
                ap.design_events.get_mut(key).unwrap().set_creator(Rc::clone(&creator));
            } else {
                ret = Err(CommonErrors::NotFound)
            }
        }

        ret
    }

    pub(crate) fn set_creator_for_shutdown_event(&mut self, creator: EventCreator, shutdown_event: Tag) -> Result<(), CommonErrors> {
        if let Some(design_event) = shutdown_event.find_in_collection(self.design_shutdown_events.iter_mut()) {
            design_event.set_creator(Rc::clone(&creator));
            Ok(())
        } else {
            Err(CommonErrors::NotFound)
        }
    }

    pub(crate) fn get_creator_for_shutdown_event(&self, shutdown_event: Tag) -> Result<EventCreator, CommonErrors> {
        if let Some(design_event) = shutdown_event.find_in_collection(self.design_shutdown_events.iter()) {
            if let Some(creator) = design_event.creator() {
                return Ok(creator);
            }
        }

        Err(CommonErrors::NotFound)
    }
}

impl Default for ProgramDatabase {
    fn default() -> Self {
        Self::new(DesignConfig::default())
    }
}

impl AsTagTrait for (SlotMapKey, &ActionData) {
    fn as_tag(&self) -> &Tag {
        &self.1.tag
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::{
        actions::action::ActionExecError,
        events::events_provider::{EventCreatorTrait, ShutdownNotifier},
        testing::OrchTestingPoller,
    };
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

    fn make_tag(val: u32) -> Tag {
        val.to_string().as_str().into()
    }

    #[test]
    fn register_event_success() {
        let pd = ProgramDatabase::default();
        let tag = make_tag(1);

        let orch_tag = pd.register_event(tag);
        assert!(orch_tag.is_ok());
        let key = *(orch_tag.unwrap().key());
        let found = pd.get_orchestration_tag(tag);
        assert_eq!(key, *(found.unwrap().key()));
    }

    #[test]
    fn register_same_event_twice() {
        let pd = ProgramDatabase::default();
        let tag = make_tag(1);

        let mut orch_tag = pd.register_event(tag);
        assert!(orch_tag.is_ok());

        orch_tag = pd.register_event(tag);
        assert!(orch_tag.is_err());
    }

    #[test]
    fn register_event_no_space_left() {
        let pd = ProgramDatabase::default();

        // Fill up the slotmap to its capacity
        for i in 0..DEFAULT_EVENTS_CAPACITY {
            let tag = make_tag(i as u32);
            let res = pd.register_event(tag);
            assert!(res.is_ok());
        }
        // Next insert should fail
        let tag = make_tag(9999);
        let res = pd.register_event(tag);
        assert_eq!(res.unwrap_err(), CommonErrors::NoSpaceLeft);
    }

    #[test]
    fn specify_event_local_success() {
        let pd = ProgramDatabase::default();

        let tag1 = make_tag(1);
        let tag2 = make_tag(2);
        pd.register_event(tag1).unwrap();
        pd.register_event(tag2).unwrap();

        struct TestEventCreator {}

        impl EventCreatorTrait for TestEventCreator {
            fn create_trigger(&mut self) -> Option<Box<dyn ActionTrait>> {
                todo!()
            }

            fn create_sync(&mut self) -> Option<Box<dyn ActionTrait>> {
                todo!()
            }

            fn create_shutdown_notifier(&mut self) -> Option<Box<dyn ShutdownNotifier>> {
                todo!()
            }
        }

        let creator: EventCreator = Rc::new(RefCell::new(TestEventCreator {}));

        let res = pd.set_creator_for_events(creator, &[tag1, tag2]);
        assert!(res.is_ok());

        // Both design events should have a creator now
        let orch1 = pd.get_orchestration_tag(tag1).unwrap();
        let orch2 = pd.get_orchestration_tag(tag2).unwrap();

        let c1 = pd
            .action_provider
            .borrow()
            .design_events
            .get(*(orch1.key()))
            .and_then(|e| e.creator().clone());

        let c2 = pd
            .action_provider
            .borrow()
            .design_events
            .get(*(orch2.key()))
            .and_then(|e| e.creator().clone());

        assert!(Rc::ptr_eq(&c1.unwrap(), &c2.unwrap()));
    }
}
