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

use async_runtime::{
    prelude::*,
    runtime::async_runtime::AsyncRuntimeBuilder,
    safety::{ensure_safety_enabled, spawn, SafetyResult},
    scheduler::execution_engine::*,
    select,
};
use foundation::prelude::*;

fn main() {
    tracing_subscriber::fmt()
        // .with_span_events(FmtSpan::FULL) // Ensures span open/close events are logged
        .with_target(false) // Optional: Remove module path
        .with_max_level(Level::DEBUG)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    let (builder, _engine_id) = AsyncRuntimeBuilder::new().with_engine(
        ExecutionEngineBuilder::new()
            .task_queue_size(256)
            .workers(4)
            .with_dedicated_worker("dedicated".into(), ThreadParameters::default())
            .enable_safety_worker(ThreadParameters::default()),
    );
    let mut runtime = builder.build().unwrap();

    let _ = runtime.block_on(async {
        ensure_safety_enabled();

        async fn short_job(result: i32) -> SafetyResult<i32, ()> {
            for _ in 0..1_000_usize {}
            Ok(result)
        }

        async fn long_job(result: i32) -> SafetyResult<i32, ()> {
            for _ in 0..1_000_000_usize {}
            Ok(result)
        }

        async fn very_long_job(result: i32) -> SafetyResult<i32, ()> {
            for _ in 0..1_000_000_000_usize {}
            Ok(result)
        }

        // First job should finish first.
        {
            info!("Test 1");

            let mut fut1 = spawn(short_job(111));
            let mut fut2 = spawn(long_job(222));
            let mut fut3 = spawn(long_job(333));
            let mut result = 0;

            select! {
                Ok(Ok(var1)) = fut1 => {
                    result = var1;
                }
                Ok(Ok(var2)) = fut2 => {
                    result = var2;
                }
                Ok(Ok(var3)) = fut3 => {
                    result = var3;
                }
            }

            assert_eq!(result, 111);
        }

        // Second job should finish first.
        {
            info!("Test 2");

            let mut fut1 = spawn(long_job(111));
            let mut fut2 = spawn(short_job(222));
            let mut fut3 = spawn(long_job(333));
            let mut result = 0;

            select! {
                Ok(Ok(var1)) = fut1 => {
                    result = var1;
                }
                Ok(Ok(var2)) = fut2 => {
                    result = var2;
                }
                Ok(Ok(var3)) = fut3 => {
                    result = var3;
                }
            }

            assert_eq!(result, 222);
        }

        // First job should finish first, but it won't match, so the third job should match first.
        {
            info!("Test 3");

            let mut fut1 = spawn(short_job(111));
            let mut fut2 = spawn(very_long_job(222));
            let mut fut3 = spawn(long_job(333));
            let mut result = 0;

            select! {
                Ok(Err(())) = fut1 => {
                    result = 111;
                }
                Ok(Ok(var2)) = fut2 => {
                    result = var2;
                }
                Ok(Ok(var3)) = fut3 => {
                    result = var3;
                }
            }

            assert_eq!(result, 333);
        }

        Ok(0)
    });
}
