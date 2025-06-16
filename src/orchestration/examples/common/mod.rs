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

use async_runtime::futures::yield_now;
use foundation::prelude::*;
use orchestration::prelude::ActionResult;

use orchestration::actions::internal::invoke::InvokeResult;

/// emulate some sleep as workaround until sleep is supported in runtime
pub fn busy_sleep() -> ActionResult {
    info!("Start sleeping");
    let mut ctr = 1000000;
    while ctr > 0 {
        ctr -= 1;
    }
    info!("End sleeping");
    Ok(())
}

pub async fn wait_ends() -> ActionResult {
    info!("Test_Event_1 triggered");
    Ok(())
}

pub async fn wait_ends2() -> ActionResult {
    info!("Test_Event_2 triggered");
    Ok(())
}

pub async fn wait_ends3() -> ActionResult {
    info!("Test_Event_3 triggered");
    Ok(())
}

pub async fn test1_func() -> ActionResult {
    info!("Start of 'test1' function.");
    info!("'test1' function yielding....");
    // yield for other tasks to run.
    yield_now::yield_now().await;
    info!("'test1' function resuming....");
    let rv = busy_sleep();
    info!("End of 'test1' function.");
    return rv;
}

pub async fn test2_func() -> ActionResult {
    info!("Start of 'test2' function.");
    let rv = busy_sleep();
    info!("End of 'test2' function.");
    return rv;
}

pub async fn test3_func() -> ActionResult {
    info!("Start of 'test3' function.");
    info!("'test3' function yielding....");
    // yield for other tasks to run.
    yield_now::yield_now().await;
    info!("'test3' function resuming....");
    let rv = busy_sleep();
    info!("End of 'test3' function.");
    return rv;
}

pub async fn test4_func() -> ActionResult {
    info!("Start of 'test4' function.");
    let rv = busy_sleep();
    info!("End of 'test4' function.");
    return rv;
}

pub fn test1_sync_func() -> InvokeResult {
    info!("Start of 'test1_sync_func' function.");
    let _ = busy_sleep();
    info!("End of 'test1_sync_func' function.");
    Ok(())
}

pub fn test2_sync_func() -> InvokeResult {
    info!("Start of 'test2_sync_func' function.");
    let _ = busy_sleep();
    info!("End of 'test2_sync_func' function.");
    Ok(())
}

pub fn test3_sync_func() -> InvokeResult {
    info!("Start of 'test3_sync_func' function.");
    let _ = busy_sleep();
    info!("End of 'test3_sync_func' function.");
    Ok(())
}
