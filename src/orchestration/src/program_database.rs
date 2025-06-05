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

use crate::actions::*;
use crate::common::orch_tag::{MapIdentifier, OrchestrationTag, OrchestrationTagNotClonable};
use crate::common::tag::Tag;
use crate::common::DesignConfig;
use crate::prelude::ActionTrait;
use foundation::{not_recoverable_error, prelude::*};
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use std::rc::Rc;
use std::{
    boxed::Box,
    cell::RefCell,
    fmt::Debug,
    future::Future,
    sync::{Arc, Mutex},
};

struct TaggedSlotMapEntry<D> {
    tag: Tag,
    data: D,
}

pub(crate) struct ActionProvider {
    clonable_invokes: SlotMap<TaggedSlotMapEntry<Box<dyn action::ClonableActionTrait>>>,
    // This map holds an option, because there's currently no API in the SlotMap to move a value out of the map.
    not_clonable_invokes: SlotMap<TaggedSlotMapEntry<Option<Box<dyn action::ActionTrait>>>>,
}

impl ActionProvider {
    pub(crate) fn new(clonable_invokes_capacity: usize, not_clonable_invokes_capacity: usize) -> Self {
        Self {
            clonable_invokes: SlotMap::new(clonable_invokes_capacity),
            not_clonable_invokes: SlotMap::new(not_clonable_invokes_capacity),
        }
    }

    pub(crate) fn return_not_clonable_data(&mut self, orch_tag: OrchestrationTag, returned_action: Box<dyn ActionTrait>) {
        match orch_tag.map_identifier() {
            MapIdentifier::NotClonableInvokeMap => {
                if !self.not_clonable_invokes.insert_at(
                    *orch_tag.key(),
                    TaggedSlotMapEntry {
                        tag: *orch_tag.tag(),
                        data: Some(returned_action),
                    },
                ) {
                    not_recoverable_error!("Failed to return a not clonable invoke action from an OrchestrationTagNotClonable.");
                }
            }
            _ => not_recoverable_error!("Tried to drop an OrchestrationTagNotClonable with an unsupported map_identifier."),
        };
    }

    pub(crate) fn provide_invoke(&mut self, key: SlotMapKey) -> Option<Box<dyn action::ActionTrait>> {
        if let Some(entry) = self.clonable_invokes.get(key) {
            Some(entry.data.clone_boxed().into_boxed_action())
        } else if let Some(optional_entry) = self.not_clonable_invokes.get_mut(key) {
            optional_entry.data.take()
        } else {
            None
        }
    }

    fn is_tag_unique(&self, tag: &Tag) -> bool {
        fn is_tag_in_map<T>(map: &SlotMap<TaggedSlotMapEntry<T>>, tag: &Tag) -> bool {
            map.iter().any(|(_, entry)| entry.tag == *tag)
        }

        !(is_tag_in_map(&self.clonable_invokes, tag) || is_tag_in_map(&self.not_clonable_invokes, tag))
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
            action_provider: Rc::new(RefCell::new(ActionProvider::new(
                params.db_params.clonable_invokes_capacity,
                params.db_params.not_clonable_invokes_capacity,
            ))),
        }
    }

    /// Registers a function as an invoke action that can be created multiple times.
    pub fn register_invoke_fn(&self, tag: Tag, action: invoke::FunctionType) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(TaggedSlotMapEntry {
                tag,
                data: invoke::Invoke::from_fn(action),
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

    /// Registers an async function as an invoke action that only be created once.
    pub fn register_invoke_async<A, F>(&self, tag: Tag, action: A) -> Result<OrchestrationTagNotClonable, CommonErrors>
    where
        A: FnMut() -> F + 'static + Send,
        F: Future<Output = action::ActionResult> + 'static + Send,
    {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.not_clonable_invokes.insert(TaggedSlotMapEntry { tag, data: None }) {
                Ok(OrchestrationTagNotClonable::new(
                    OrchestrationTag::new(tag, key, MapIdentifier::NotClonableInvokeMap, Rc::clone(&self.action_provider)),
                    invoke::Invoke::from_async(action),
                ))
            } else {
                Err(CommonErrors::NoSpaceLeft)
            }
        } else {
            Err(CommonErrors::AlreadyDone)
        }
    }

    /// Registers an async function as an invoke action that can be created multiple times.
    pub fn register_invoke_async_clonable<A, F>(&self, tag: Tag, action: A) -> Result<OrchestrationTag, CommonErrors>
    where
        A: FnMut() -> F + 'static + Send + Clone,
        F: Future<Output = action::ActionResult> + 'static + Send + Clone,
    {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(TaggedSlotMapEntry {
                tag,
                data: invoke::Invoke::from_async_clonable(action),
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

    /// Registers a method on an object as an invoke action that can only be created once.
    pub fn register_invoke_method<T: 'static + Send>(
        &self,
        tag: Tag,
        obj: Arc<Mutex<T>>,
        method: fn(&mut T) -> action::ActionResult,
    ) -> Result<OrchestrationTag, CommonErrors> {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(TaggedSlotMapEntry {
                tag,
                data: invoke::Invoke::from_arc(obj, method),
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

    /// Registers a method on a shared object as an invoke action that can only be created once.
    pub fn register_invoke_method_async<T: 'static + Send, F, Fut>(
        &self,
        tag: Tag,
        obj: Arc<Mutex<T>>,
        method: F,
    ) -> Result<OrchestrationTagNotClonable, CommonErrors>
    where
        F: FnMut(Arc<Mutex<T>>) -> Fut + 'static + Send,
        Fut: Future<Output = action::ActionResult> + 'static + Send,
    {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.not_clonable_invokes.insert(TaggedSlotMapEntry { tag, data: None }) {
                Ok(OrchestrationTagNotClonable::new(
                    OrchestrationTag::new(tag, key, MapIdentifier::NotClonableInvokeMap, Rc::clone(&self.action_provider)),
                    invoke::Invoke::from_arc_mtx(obj, method),
                ))
            } else {
                Err(CommonErrors::NoSpaceLeft)
            }
        } else {
            Err(CommonErrors::AlreadyDone)
        }
    }

    /// Registers a method on a shared object as an invoke action that can only be created once.
    pub fn register_invoke_method_async_clonable<T: 'static + Send, F, Fut>(
        &self,
        tag: Tag,
        obj: Arc<Mutex<T>>,
        method: F,
    ) -> Result<OrchestrationTag, CommonErrors>
    where
        F: FnMut(Arc<Mutex<T>>) -> Fut + 'static + Send + Clone,
        Fut: Future<Output = action::ActionResult> + 'static + Send + Clone,
    {
        let mut ap = self.action_provider.borrow_mut();

        if ap.is_tag_unique(&tag) {
            if let Some(key) = ap.clonable_invokes.insert(TaggedSlotMapEntry {
                tag,
                data: invoke::Invoke::from_arc_mtx_clonable(obj, method),
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

    /// Returns an `OrchestrationTagNotClonable` for an action previously registered with the given tag if the tag was not already used.
    pub fn get_orchestration_tag_not_clonable(&self, tag: Tag) -> Option<OrchestrationTagNotClonable> {
        let map = &mut self.action_provider.borrow_mut().not_clonable_invokes;

        if let Some((key, _)) = map.iter().find(|(_, entry)| entry.tag == tag) {
            // A mutable borrow is needed to take the data out of the entry, but iter_mut is not implemented for SlotMap.
            if let Some(entry) = map.get_mut(key) {
                if let Some(data) = entry.data.take() {
                    return Some(OrchestrationTagNotClonable::new(
                        OrchestrationTag::new(entry.tag, key, MapIdentifier::NotClonableInvokeMap, Rc::clone(&self.action_provider)),
                        data,
                    ));
                }
            }
        }

        None
    }
}

impl Default for ProgramDatabase {
    fn default() -> Self {
        Self::new(DesignConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::action::ActionResult;
    use super::*;
    use foundation::prelude::CommonErrors;
    use std::task;

    #[test]
    fn create_invoke_fn_action() {
        let pd = ProgramDatabase::default();

        fn test_invoke() -> ActionResult {
            Err(CommonErrors::GenericError)
        }

        let tag = pd.register_invoke_fn(Tag::from_str_static("tag1"), test_invoke).unwrap();
        let mut invoke = invoke::Invoke::from_tag(&tag);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());

        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));
    }

    #[test]
    fn create_invoke_fn_action_after_get() {
        let pd = ProgramDatabase::default();

        fn test_invoke() -> ActionResult {
            Err(CommonErrors::GenericError)
        }

        let tag = Tag::from_str_static("tag1");
        assert!(pd.register_invoke_fn(tag, test_invoke).is_ok());
        let orch_tag = pd.get_orchestration_tag(tag).unwrap();

        let mut invoke = invoke::Invoke::from_tag(&orch_tag);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());

        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));
    }

    #[test]
    fn try_to_register_the_same_tag_twice() {
        let pd = ProgramDatabase::default();

        fn test_invoke() -> ActionResult {
            Err(CommonErrors::GenericError)
        }

        let tag = Tag::from_str_static("tag1");

        let orch_tag = pd.register_invoke_fn(tag, test_invoke).unwrap();
        let mut invoke = invoke::Invoke::from_tag(&orch_tag);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());

        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));
        assert_eq!(pd.register_invoke_fn(tag, test_invoke).err(), Some(CommonErrors::AlreadyDone));
    }

    #[test]
    fn create_multiple_invoke_fn_actions() {
        let pd = ProgramDatabase::default();

        fn test_invoke1() -> ActionResult {
            Err(CommonErrors::GenericError)
        }

        let tag1 = pd.register_invoke_fn(Tag::from_str_static("tag1"), test_invoke1).unwrap();

        let mut invoke = invoke::Invoke::from_tag(&tag1);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));

        let mut invoke = invoke::Invoke::from_tag(&tag1);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));
    }

    #[test]
    fn create_different_invoke_fn_actions() {
        let pd = ProgramDatabase::default();

        fn test_invoke1() -> ActionResult {
            Err(CommonErrors::GenericError)
        }

        fn test_invoke2() -> ActionResult {
            Err(CommonErrors::Timeout)
        }

        let tag1 = pd.register_invoke_fn(Tag::from_str_static("tag1"), test_invoke1).unwrap();
        let tag2 = pd.register_invoke_fn(Tag::from_str_static("tag2"), test_invoke2).unwrap();

        let mut invoke = invoke::Invoke::from_tag(&tag1);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));

        let mut invoke = invoke::Invoke::from_tag(&tag2);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::Timeout)));
    }

    #[test]
    fn multiple_provide_for_invoke_async_action() {
        let pd = ProgramDatabase::default();
        let t = Tag::from_str_static("tag1");

        let ot = pd.register_invoke_async(t, async || Err(CommonErrors::GenericError)).unwrap();

        assert!(pd.get_orchestration_tag(t).is_none());
        assert!(pd.get_orchestration_tag_not_clonable(t).is_none());

        let mut invoke = invoke::Invoke::from_tag_not_clonable(ot);
        let mut f = invoke.execute();
        let mut c = task::Context::from_waker(task::Waker::noop());

        assert_eq!(f.as_mut().poll(&mut c), task::Poll::Ready(Err(CommonErrors::GenericError)));
    }

    #[test]
    fn not_clonable_orchestration_tag_returns_action_on_drop() {
        let pd = ProgramDatabase::default();
        let t = Tag::from_str_static("tag1");

        let ot = pd.register_invoke_async(t, async || Err(CommonErrors::GenericError)).unwrap();
        assert!(pd.get_orchestration_tag(t).is_none());
        assert!(pd.get_orchestration_tag_not_clonable(t).is_none());
        drop(ot);

        let ot = pd.get_orchestration_tag_not_clonable(t).unwrap();
        assert!(pd.get_orchestration_tag(t).is_none());
        assert!(pd.get_orchestration_tag_not_clonable(t).is_none());

        let mut invoke = invoke::Invoke::from_tag_not_clonable(ot);
        let mut f = invoke.execute();
        let mut c = task::Context::from_waker(task::Waker::noop());

        assert_eq!(f.as_mut().poll(&mut c), task::Poll::Ready(Err(CommonErrors::GenericError)));
    }

    #[test]
    fn create_multiple_invoke_method_actions() {
        let pd = ProgramDatabase::default();

        struct TestObject {}

        impl TestObject {
            fn test_method(&mut self) -> ActionResult {
                Err(CommonErrors::GenericError)
            }
        }

        let tag = pd
            .register_invoke_method(Tag::from_str_static("tag1"), Arc::new(Mutex::new(TestObject {})), TestObject::test_method)
            .unwrap();

        let mut invoke = invoke::Invoke::from_tag(&tag);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));

        let mut invoke = invoke::Invoke::from_tag(&tag);
        let mut future = invoke.execute();
        let mut context = task::Context::from_waker(task::Waker::noop());
        assert_eq!(future.as_mut().poll(&mut context), task::Poll::Ready(Err(CommonErrors::GenericError)));
    }
}
