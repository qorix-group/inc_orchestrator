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

use std::sync::atomic::{AtomicU32, Ordering};

use async_runtime::futures::{sleep, yield_now};
use foundation::prelude::*;
use orchestration::actions::action::UserErrValue;
use orchestration::actions::invoke::InvokeResult;
use orchestration::api::design::Design;

/// emulate some sleep as workaround until sleep is supported in runtime
pub fn busy_sleep() -> InvokeResult {
    info!("Start sleeping");
    let mut ctr = 1000000;
    while ctr > 0 {
        ctr -= 1;
    }
    info!("End sleeping");
    Ok(())
}

pub async fn wait_ends() -> InvokeResult {
    info!("Test_Event_1 triggered");
    Ok(())
}

pub async fn wait_ends2() -> InvokeResult {
    info!("Test_Event_2 triggered");
    Ok(())
}

pub async fn wait_ends3() -> InvokeResult {
    info!("Test_Event_3 triggered");
    Ok(())
}

pub async fn test1_async_func() -> InvokeResult {
    info!("Start of 'test1_async_func' function.");
    info!("'test1_async_func' function yielding....");
    // yield for other tasks to run.
    yield_now::yield_now().await;
    info!("'test1_async_func' function resuming....");
    let rv = busy_sleep();
    info!("End of 'test1_async_func' function.");
    return rv;
}

pub async fn test2_async_func() -> InvokeResult {
    info!("Start of 'test2_async_func' function.");
    let rv = busy_sleep();
    info!("End of 'test2_async_func' function.");
    return rv;
}

pub async fn test3_async_func() -> InvokeResult {
    info!("Start of 'test3_async_func' function.");
    info!("'test3_async_func' function yielding....");
    // yield for other tasks to run.
    yield_now::yield_now().await;
    info!("'test3_async_func' function resuming....");
    let rv = busy_sleep();
    info!("End of 'test3_async_func' function.");
    return rv;
}

pub async fn test4_async_func() -> InvokeResult {
    info!("Start of 'test4_async_func' function.");
    sleep::sleep(std::time::Duration::from_millis(10)).await;
    info!("End of 'test4_async_func' function.");
    Ok(())
}

pub fn test1_sync_func() -> InvokeResult {
    info!("Start of 'test1_sync_func' function.");

    info!("End of 'test1_sync_func' function.");
    Ok(())
}

pub fn test2_sync_func() -> InvokeResult {
    info!("Start of 'test2_sync_func' function.");

    info!("End of 'test2_sync_func' function.");
    Ok(())
}

pub fn test3_sync_func() -> InvokeResult {
    info!("Start of 'test3_sync_func' function.");

    info!("End of 'test3_sync_func' function.");
    Ok(())
}

pub fn test4_sync_func() -> InvokeResult {
    info!("Start of 'test4_sync_func' function.");

    info!("End of 'test4_sync_func' function.");
    Ok(())
}

pub fn always_produce_error() -> InvokeResult {
    error!("Executed 'always_produce_error' function.");
    UserErrValue::from(123).into()
}

pub fn error_after_third_run() -> InvokeResult {
    static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

    let count = CALL_COUNT.fetch_add(1, Ordering::Relaxed) + 1;

    if count > 3 {
        error!("Executed 'error_after_third_run' function  with count {}", count);
        UserErrValue::from(3456).into()
    } else {
        info!("Executed 'error_after_third_run' function with count {}", count);
        Ok(())
    }
}

pub fn register_all_common_into_design(design: &mut Design) -> Result<(), CommonErrors> {
    design.register_invoke_fn("test1_sync_func".into(), test1_sync_func)?;
    design.register_invoke_fn("test2_sync_func".into(), test2_sync_func)?;
    design.register_invoke_fn("test3_sync_func".into(), test3_sync_func)?;
    design.register_invoke_fn("test4_sync_func".into(), test4_sync_func)?;
    design.register_invoke_async("test1_async_func".into(), test1_async_func)?;
    design.register_invoke_async("test2_async_func".into(), test2_async_func)?;
    design.register_invoke_async("test3_async_func".into(), test3_async_func)?;
    design.register_invoke_async("test4_async_func".into(), test4_async_func)?;
    design.register_invoke_fn("always_produce_error".into(), always_produce_error)?;
    design.register_invoke_fn("error_after_third_run".into(), error_after_third_run)?;

    design.register_event("Event1".into())?;
    design.register_event("Event2".into())?;
    design.register_event("Event3".into())?;
    design.register_event("Event4".into())?;

    Ok(())
}
