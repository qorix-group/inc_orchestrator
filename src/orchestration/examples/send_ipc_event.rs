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
use logging_tracing::TracingLibraryBuilder;
use orchestration::{
    api::{design::Design, Orchestration},
    common::{tag::Tag, DesignConfig},
    prelude::*,
};
use std::env;

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());

    design.register_event(Tag::from_str_static("InternalEvent"))?;

    // Create a program with trigger action
    design.add_program("ExampleProgram", move |design_instance, builder| {
        builder.with_run_action(TriggerBuilder::from_design("InternalEvent", &design_instance));

        Ok(())
    });

    Ok(design)
}

fn main() {
    // Collect command-line arguments
    let args: std::vec::Vec<String> = env::args().collect();

    // Check if a string argument is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <event_name>", args[0]);
        return;
    }

    // Get the event name from the command-line arguments
    let event = &args[1];
    // Setup any logging framework you want to use.
    let mut logger = TracingLibraryBuilder::new().global_log_level(Level::INFO).enable_logging(true).build();

    logger.init_log_trace();

    // Create runtime
    let (builder, _engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(1));

    let mut runtime = builder.build().unwrap();

    // Build Orchestration
    let mut orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // Deployment part - specify event details
    let mut deployment = orch.get_deployment_mut();

    // Bind design event to the system event
    deployment
        .bind_events_as_global(event.as_str(), &["InternalEvent".into()])
        .expect("Failed to specify event");

    // Create programs
    let mut programs = orch.create_programs().unwrap();

    // Put programs into runtime and run them
    let _ = runtime.block_on(async move {
        let _ = programs.programs.pop().unwrap().run_n(1).await;
        debug!("Program finished running.");
        Ok(0)
    });

    info!("Successfully sent IPC event: {}", event);
}
