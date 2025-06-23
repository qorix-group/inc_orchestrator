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
use std::cell::RefCell;
use std::rc::Rc;

use crate::events::event_traits::IpcProvider;
use crate::events::GlobalEventProvider;
use crate::{
    actions::internal::{sync::Sync, trigger::Trigger},
    common::tag::AsTagTrait,
};
use foundation::prelude::*;
use iceoryx2_bb_container::slotmap::SlotMapKey;

use crate::{actions::internal::action::ActionTrait, common::tag::Tag, events::local_events::LocalEvent};

pub const DEFAULT_EVENTS_CAPACITY: usize = 256;

#[derive(Copy, Clone, Debug, PartialEq)]
enum EventType {
    /// Event that is process local
    Local,

    /// Even that is cross processes
    Global,
}

///
/// Provides real events into design and allows to specify which design events should map to it.
///
pub struct EventsProvider<GlobalProvider = GlobalEventProvider>
where
    GlobalProvider: IpcProvider,
{
    events: Vec<DeploymentEventInfo>,
    local_event_next_id: u64,
    ipc: Rc<RefCell<GlobalProvider>>,
}

impl<GlobalProvider: IpcProvider + 'static> Default for EventsProvider<GlobalProvider> {
    fn default() -> Self {
        Self::new()
    }
}

impl<GlobalProvider: IpcProvider + 'static> EventsProvider<GlobalProvider> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(DEFAULT_EVENTS_CAPACITY),
            local_event_next_id: 0,
            ipc: Rc::new(RefCell::new(GlobalProvider::new())),
        }
    }

    /// Deployment time event specification
    /// This let integrator register new event and specify whether it's local or global and which design events should map to it.
    pub(crate) fn specify_global_event(&mut self, system_event: &str) -> Result<EventCreator, CommonErrors> {
        self.specify_event(system_event, EventType::Global)
    }

    pub(crate) fn specify_local_event(&mut self) -> Result<EventCreator, CommonErrors> {
        let name = format!("local_event_{}", self.local_event_next_id);
        self.local_event_next_id += 1;

        self.specify_event(name.as_str(), EventType::Local)
    }

    fn specify_event(&mut self, system_event: &str, typ: EventType) -> Result<EventCreator, CommonErrors> {
        let system_event_tag: Tag = system_event.into();

        if system_event_tag.is_in_collection(self.events.iter()) {
            return Err(CommonErrors::AlreadyDone);
        }

        let creator = self.choose_creator(typ, &system_event_tag, system_event);

        self.events.push(DeploymentEventInfo {
            system_tag: system_event_tag,
            creator: Rc::clone(&creator),
        });

        Ok(creator)
    }

    #[allow(dead_code)]
    pub(crate) fn get_event_creator(&self, system_event: &str) -> Option<EventCreator> {
        Some(Rc::clone(
            &Into::<Tag>::into(system_event).find_in_collection(self.events.iter())?.creator,
        ))
    }

    fn choose_creator(&self, typ: EventType, tag: &Tag, system_event: &str) -> EventCreator {
        match typ {
            EventType::Local => Self::create_local_event_action(*tag),
            EventType::Global => Self::create_global_event_action(Rc::clone(&self.ipc), system_event),
        }
    }

    fn create_local_event_action(tracking_tag: Tag) -> EventCreator {
        let mut evt = LocalEvent::new();

        Rc::new(RefCell::new(move |typ: EventActionType| match typ {
            EventActionType::Sync => Some(Sync::new(evt.get_listener()?) as Box<dyn ActionTrait>),
            EventActionType::Trigger => {
                let n = evt.get_notifier();
                if n.is_none() {
                    debug!(
                        "Failed to create Trigger Action, notifier is None. Did you tried to create two notifiers for the same event ({:?})?",
                        tracking_tag
                    );
                }
                Some(Trigger::new(n?) as Box<dyn ActionTrait>)
            }
        }))
    }

    fn create_global_event_action(ipc: Rc<RefCell<GlobalProvider>>, system_event: &str) -> EventCreator {
        let event = system_event.to_string();
        Rc::new(RefCell::new(move |typ: EventActionType| match typ {
            EventActionType::Sync => Some(Sync::new(ipc.borrow_mut().get_listener(event.as_str())?) as Box<dyn ActionTrait>),
            EventActionType::Trigger => Some(Trigger::new(ipc.borrow_mut().get_notifier(event.as_str())?) as Box<dyn ActionTrait>),
        }))
    }
}

pub(crate) enum EventActionType {
    Sync,
    Trigger,
}

pub(crate) type EventCreator = Rc<RefCell<dyn FnMut(EventActionType) -> Option<Box<dyn ActionTrait>>>>;

pub(crate) struct DesignEvent {
    tag: Tag,
    creator: Option<EventCreator>,
}

impl DesignEvent {
    pub fn new(tag: Tag) -> Self {
        Self { tag, creator: None }
    }

    pub fn creator(&self) -> Option<EventCreator> {
        self.creator.clone()
    }

    pub fn set_creator(&mut self, creator: EventCreator) {
        let prev = self.creator.replace(creator);
        if prev.is_some() {
            warn!(
                "Event with tag {:?} already has a binding, we replace it with new one provided.",
                self.tag
            );
        }
    }
}

struct DeploymentEventInfo {
    system_tag: Tag,

    #[allow(dead_code)]
    creator: EventCreator, // EventType is bind into this
}

impl AsTagTrait for &DeploymentEventInfo {
    fn as_tag(&self) -> &Tag {
        &self.system_tag
    }
}

impl AsTagTrait for (SlotMapKey, &DesignEvent) {
    fn as_tag(&self) -> &Tag {
        &self.1.tag
    }
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::testing::OrchTestingPoller;
    use foundation::prelude::CommonErrors;
    use testing::assert_poll_ready;

    #[test]
    fn new_provider() {
        let provider: EventsProvider = EventsProvider::new();
        assert_eq!(provider.events.len(), 0);
    }

    #[test]
    fn specify_event_duplicate() {
        let mut provider: EventsProvider = EventsProvider::new();

        provider.specify_event("100", EventType::Local).unwrap();
        // Try to specify again with the same system tag
        let res = provider.specify_event("100", EventType::Local);
        assert_eq!(res.err().unwrap(), CommonErrors::AlreadyDone);
    }

    #[test]
    fn creating_same_trigger_action_twice_causes_fail() {
        let mut provider: EventsProvider = EventsProvider::new();

        let res = provider.specify_event("100", EventType::Local);
        assert!(res.is_ok());

        let creator = provider.get_event_creator("100").unwrap();

        let trigger_action = creator.borrow_mut()(EventActionType::Trigger);

        assert!(trigger_action.is_some());
        assert!(creator.borrow_mut()(EventActionType::Trigger).is_none());
    }

    #[test]
    fn creating_same_sync_action_n_times_works() {
        let mut provider: EventsProvider = EventsProvider::new();

        let res = provider.specify_event("100", EventType::Local);
        assert!(res.is_ok());

        let creator = provider.get_event_creator("100").unwrap();

        let mut trigger_action = creator.borrow_mut()(EventActionType::Sync);
        assert!(trigger_action.is_some());

        trigger_action = creator.borrow_mut()(EventActionType::Sync);
        assert!(trigger_action.is_some());

        trigger_action = creator.borrow_mut()(EventActionType::Sync);
        assert!(trigger_action.is_some());
    }

    #[test]
    fn sync_trigger_local_pair_works() {
        let mut provider: EventsProvider = EventsProvider::new();

        let res = provider.specify_event("100", EventType::Local);
        assert!(res.is_ok());

        let mut trigger_action = provider.get_event_creator("100").unwrap().borrow_mut()(EventActionType::Trigger).unwrap();
        let mut sync_action = provider.get_event_creator("100").unwrap().borrow_mut()(EventActionType::Sync).unwrap();

        let trig_f = trigger_action.try_execute().unwrap();

        let sync_f = sync_action.try_execute().unwrap();

        let mut sync_poller = OrchTestingPoller::new(sync_f);
        let mut trigger_poller = OrchTestingPoller::new(trig_f);

        let mut ret = sync_poller.poll();
        assert!(ret.is_pending()); // Sync should be pending as no trigger has been called yet

        ret = trigger_poller.poll(); // Call trigger

        assert_poll_ready(ret, Ok(()));

        ret = sync_poller.poll(); // Now sync should be ready as trigger was called
        assert_poll_ready(ret, Ok(()));
    }

    #[test]
    fn sync_trigger_local_from_different_tag_does_not_unblock() {
        let mut provider: EventsProvider = EventsProvider::new();

        let mut res = provider.specify_event("100", EventType::Local);
        assert!(res.is_ok());
        res = provider.specify_event("101", EventType::Local);
        assert!(res.is_ok());

        let mut trigger_action = provider.get_event_creator("100").unwrap().borrow_mut()(EventActionType::Trigger).unwrap();

        let mut sync_action = provider.get_event_creator("101").unwrap().borrow_mut()(EventActionType::Sync).unwrap();

        let trig_f = trigger_action.try_execute().unwrap();

        let sync_f = sync_action.try_execute().unwrap();

        let mut sync_poller = OrchTestingPoller::new(sync_f);
        let mut trigger_poller = OrchTestingPoller::new(trig_f);

        let mut ret = sync_poller.poll();
        assert!(ret.is_pending()); // Sync should be pending as no trigger has been called yet

        ret = trigger_poller.poll(); // Call trigger

        assert_poll_ready(ret, Ok(()));

        ret = sync_poller.poll();
        assert!(ret.is_pending()); // Sync should be pending as  trigger was called for different event
    }
}
