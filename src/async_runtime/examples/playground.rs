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

use async_runtime::prelude::*;
use async_runtime::{
    runtime::async_runtime::AsyncRuntimeBuilder,
    safety::{self, ensure_safety_enabled},
    scheduler::execution_engine::*,
    spawn, spawn_on_dedicated,
};

use ::core::future::Future;
use foundation::prelude::*;

pub struct X {}

impl Future for X {
    type Output = ();

    fn poll(self: ::core::pin::Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<Self::Output> {
        cx.waker().wake_by_ref();

        cx.waker().wake_by_ref();

        cx.waker().wake_by_ref();

        cx.waker().wake_by_ref();

        cx.waker().wake_by_ref();

        cx.waker().wake_by_ref();

        cx.waker().wake_by_ref();

        ::core::task::Poll::Ready(())
    }
}

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
            .workers(3)
            .with_dedicated_worker("dedicated".into(), ThreadParameters::default())
            .enable_safety_worker(ThreadParameters::default()),
    );
    let mut runtime = builder.build().unwrap();

    let _ = runtime.block_on(async {
        ensure_safety_enabled();
        // TASK
        error!("We do have first enter into runtime ;)");

        let handle = spawn(async {
            // TASK
            error!("And again from one we are in another ;)");

            let _ = safety::spawn(async {
                //TASK
                error!("And again from one nested from dedicated ;)");

                Err(0) as Result<i32, i32>
            })
            .await
            .unwrap()
            .is_err();

            let x = "dedicated".into();

            error!("I RUN FROM SAFETY FROM NOW ON !!!");

            spawn_on_dedicated(
                async {
                    // TASK
                    error!("I AM DEDICATED  ;)");

                    error!("I AM DEDICATED RESTORED  ;)");
                    1
                },
                x,
            )
            .await
            .unwrap();

            1
        });

        let res = handle.await;
        error!("After await res is {}", res.unwrap());

        let x = X {};

        x.await;
        error!("After multi waker");
        Ok(0)
    });
}
