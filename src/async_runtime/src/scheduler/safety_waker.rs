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

use super::task::async_task::*;
use core::task::{RawWaker, RawWakerVTable, Waker};

fn clone_waker(data: *const ()) -> RawWaker {
    let task_header_ptr = data as *const TaskHeader;
    let task_ref = unsafe { TaskRef::from_raw(task_header_ptr) };

    let new_waker = task_ref.clone();

    ::core::mem::forget(task_ref); // We need to make sure the instance from which we clone, is forgotten since we did not consumed it, it was only bring back for a moment to clone

    let raw = TaskRef::into_raw(new_waker);
    RawWaker::new(raw as *const (), &VTABLE)
}

fn wake(data: *const ()) {
    let task_header_ptr = data as *const TaskHeader;
    let task_ref = unsafe { TaskRef::from_raw(task_header_ptr) };

    task_ref.schedule_safety();

    drop(task_ref); // wake uses move semantic, so we are owner of data now, so we need to cleanup
}

fn wake_by_ref(data: *const ()) {
    let task_header_ptr = data as *const TaskHeader;
    let task_ref = unsafe { TaskRef::from_raw(task_header_ptr) };

    task_ref.schedule_safety();

    ::core::mem::forget(task_ref); // don't touch refcount from our data since this is done by drop_waker
}

fn drop_waker(data: *const ()) {
    let task_header_ptr = data as *const TaskHeader;
    let task_ref = unsafe { TaskRef::from_raw(task_header_ptr) };

    drop(task_ref); // We get rid of instance, so ref count will be decremented
}

// Define the RawWakerVTable
static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

///
/// Waker will store internally a pointer to the ref counted Task.
///
pub(crate) unsafe fn create_safety_waker(waker: Waker) -> Waker {
    let raw_waker = RawWaker::new(waker.data(), &VTABLE);

    // Forget original as we took over the ownership, so ref count
    ::core::mem::forget(waker);

    // Convert RawWaker to Waker
    unsafe { Waker::from_raw(raw_waker) }
}
