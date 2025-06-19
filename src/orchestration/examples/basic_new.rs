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
    actions::internal::prelude::*,
    api::{design::Design, Orchestration},
    common::{tag::Tag, DesignConfig},
};

mod common;
use common::{test1_sync_func, test2_sync_func, test3_sync_func};

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("test1".into(), test1_sync_func)?;
    let t2_tag = design.register_invoke_fn("test2".into(), test2_sync_func)?;
    design.register_invoke_fn("test3".into(), test3_sync_func)?;

    let evt1 = design.register_event(Tag::from_str_static("Event1"))?;
    let evt2 = design.register_event(Tag::from_str_static("Event2"))?;
    // Create a program with some actions

    design.add_program("ExampleDesignProgram", move |design_instance, builder| {
        let t3_tag = design_instance.get_orchestration_tag("test3".into())?;

        builder.with_body(
            SequenceBuilder::new()
                .with_step(TriggerBuilder::from_design("Event1", &design_instance))
                .with_step(Invoke::from_tag(&t1_tag))
                .with_step(Invoke::from_tag(&t2_tag))
                .with_step(Invoke::from_tag(&t2_tag))
                .with_step(SyncBuilder::from_tag(&evt1))
                .with_step(SyncBuilder::from_tag(&evt2))
                .with_step(Invoke::from_tag(&t3_tag))
                .build(),
        );

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

    // Mark user events as local one.
    deployment
        .bind_events_as_local(&["Event1".into(), "Event2".into()])
        .expect("Failed to specify event");

    // Create programs
    let mut programs = orch.create_programs().unwrap();

    // Put programs into runtime and run them
    let _ = runtime.block_on(async move {
        let _ = programs.programs.pop().unwrap().run_n(1).await;
        info!("Program finished running.");
        Ok(0)
    });

    info!("Exit.");
}
