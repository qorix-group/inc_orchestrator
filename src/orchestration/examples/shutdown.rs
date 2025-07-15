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

use async_runtime::{runtime::async_runtime::AsyncRuntimeBuilder, scheduler::execution_engine::*};
use foundation::prelude::*;
use logging_tracing::{TraceScope, TracingLibraryBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::Invoke,
};
use std::{thread, time::Duration};

mod common;

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());
    let run_tag = design.register_invoke_async("PendingIndefinitely".into(), async || ::core::future::pending().await)?;
    let start_tag = design.register_invoke_fn("StartAction".into(), || {
        info!("Start action executed.");
        Ok(())
    })?;
    let stop_tag = design.register_invoke_fn("StopAction".into(), || {
        info!("Stop action executed.");
        Ok(())
    })?;

    design.register_shutdown_event("ExampleShutdown".into())?;

    design.add_program("ExampleDesignProgram", move |_design, builder| {
        builder
            .with_run_action(Invoke::from_tag(&run_tag))
            .with_start_action(Invoke::from_tag(&start_tag))
            .with_stop_action(Invoke::from_tag(&stop_tag), Duration::from_secs(5))
            .with_shutdown_event("ExampleShutdown".into());

        Ok(())
    });

    Ok(design)
}

fn main() {
    // Setup any logging framework you want to use.
    let mut logger = TracingLibraryBuilder::new()
        .global_log_level(Level::DEBUG)
        .enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build();

    logger.init_log_trace();

    // Create runtime
    let (builder, _engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2));
    let mut runtime = builder.build().unwrap();

    // Build Orchestration

    let mut orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();
    deployment
        .bind_shutdown_event_as_local("ExampleShutdown".into())
        .expect("Failed to bind shutdown event");

    let mut shutdown = deployment
        .get_shutdown_notifier("ExampleShutdown".into())
        .expect("Failed to get shutdown notifier");

    // Create program
    let mut programs = orch.create_programs().unwrap();
    let mut program = programs.programs.pop().unwrap();

    // Put programs into runtime and run them
    let _ = runtime.spawn(async move {
        let _ = program.run().await;
        info!("Program finished running.");
        Ok(0)
    });

    info!("Runtime spawned");
    thread::sleep(Duration::from_secs(5));
    info!("Calling shutdown");
    let _ = shutdown.shutdown();
    runtime.wait_for_all_engines();
    info!("Exit.");
}
