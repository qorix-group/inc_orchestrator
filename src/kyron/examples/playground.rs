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
#![allow(dead_code, unused_imports, unused_variables)]
use kyron::futures::sleep;
use kyron::ipc::iceoryx2::EventBuilderAsyncExt;
use kyron::prelude::*;
use kyron::{
    runtime::*,
    safety::{self, ensure_safety_enabled},
    spawn,
};

use ::core::future::Future;
use core::time::Duration;
use iceoryx2::prelude::*;
use kyron_foundation::prelude::*;

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

    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new().with_engine(
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

        spawn(async {
            // TASK
            error!("And again from one we are in another ;)");
            let node = NodeBuilder::new().create::<ipc_threadsafe::Service>().unwrap();

            let event = node
                .service_builder(&"MyEventNameq".try_into().unwrap())
                .event()
                .open_or_create()
                .unwrap();

            let listener = event.listener_builder().create_async().unwrap();

            let mut count = 0;
            loop {
                // info!("Waiting for Iceoryx event in batches ...");
                listener
                    .wait_all(&mut |event_id| {
                        info!("Received Iceoryx event: {}", event_id.as_value() + count * 256);
                        if event_id.as_value() == 254 {
                            count += 1;
                        }
                    })
                    .await
                    .unwrap();
            }
        });

        0
    });

    println!("sdfdsf");
}
