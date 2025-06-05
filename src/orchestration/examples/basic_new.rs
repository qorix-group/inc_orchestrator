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

use std::{thread, time::Duration};

use async_runtime::{runtime::async_runtime::AsyncRuntimeBuilder, scheduler::execution_engine::*};
use foundation::prelude::*;
use logging_tracing::{TraceScope, TracingLibraryBuilder};
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
    prelude::*,
    program::ProgramBuilder,
};

mod common;
use common::{test1_sync_func, test2_sync_func, test3_sync_func};

fn example_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("test1".into(), test1_sync_func)?;
    let t2_tag = design.register_invoke_fn("test2".into(), test2_sync_func)?;
    design.register_invoke_fn("test3".into(), test3_sync_func)?;

    // Create a program with some actions

    design.add_program("ExampleDesignProgram".into(), move |design_instance| {
        let t3_tag = design_instance.get_orchestration_tag("test3".into())?;

        Ok(ProgramBuilder::new("ProgramName")
            .with_body(
                Sequence::new()
                    .with_step(Invoke::from_tag(&t1_tag))
                    .with_step(Invoke::from_tag(&t2_tag))
                    .with_step(Invoke::from_tag(&t3_tag)),
            )
            .with_shutdown_notification(Sync::new("somename"))
            .build())
    });

    Ok(design)
}

fn main() {
    // Setup any logging framework you want to use.
    let mut logger = TracingLibraryBuilder::new()
        .global_log_level(Level::INFO)
        .enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build();

    logger.init_log_trace();

    // Create runtime
    let mut runtime = AsyncRuntimeBuilder::new()
        .with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2))
        .build()
        .unwrap();

    {
        // Start the event handling thread.
        // TODO: Will be removed soon
        Event::get_instance().lock().unwrap().create_polling_thread();
    }

    // Build Orchestration

    let orch = Orchestration::new()
        .add_design(example_component_design().expect("Failed to create design"))
        .design_done();

    // For now, no deployment

    // Create programs
    let mut programs = orch.create_programs().unwrap();

    // Put programs into runtime and run them
    let _ = runtime.enter_engine(async move {
        programs.programs.pop().unwrap().run_n(3).await;
        info!("Program finished running.");
    });

    // wait for some time to allow the engine finishes the last action
    thread::sleep(Duration::new(50, 0));
    println!("Exit.");
}
