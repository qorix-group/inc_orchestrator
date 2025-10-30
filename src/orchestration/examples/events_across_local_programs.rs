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

use kyron::runtime::*;
use kyron_foundation::prelude::*;
use logging_tracing::TracingLibraryBuilder;
use orchestration::{
    actions::{invoke::Invoke, sequence::SequenceBuilder, sync::SyncBuilder, trigger::TriggerBuilder},
    api::{design::Design, Orchestration},
    common::DesignConfig,
};

mod common;
use common::register_all_common_into_design;

fn program1_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign1".into(), DesignConfig::default());

    register_all_common_into_design(&mut design)?; // Register our common functions, events, etc

    // Create a program describing task chain
    design.add_program("ExampleDesign1", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("Event1", design_instance))
                .with_step(Invoke::from_design("test1_sync_func", design_instance))
                .build(),
        );
        Ok(())
    });

    Ok(design)
}

fn program2_component_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("ExampleDesign2".into(), DesignConfig::default());

    register_all_common_into_design(&mut design)?; // Register our common functions, events, etc

    // Create a program describing task chain
    design.add_program("ExampleDesign2", move |design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_design("test4_sync_func", design_instance))
                .with_step(TriggerBuilder::from_design("Event1", design_instance))
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
        // .enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build();

    logger.init_log_trace();

    // Create runtime
    let (builder, _engine_id) = kyron::runtime::RuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(256).workers(2));
    let mut runtime = builder.build().unwrap();

    // Build Orchestration
    let mut orch = Orchestration::new()
        .add_design(program1_component_design().expect("Failed to create design1"))
        .add_design(program2_component_design().expect("Failed to create design2"))
        .design_done();

    // Specify deployment information

    orch.get_deployment_mut()
        .bind_events_as_local(&["Event1".into()])
        .expect("Failed to specify event");

    // Create programs
    let mut program_manager = orch.into_program_manager().unwrap();
    let mut programs = program_manager.get_programs();

    // Put programs into runtime and run them
    runtime.block_on(async move {
        let mut program1 = programs.pop().unwrap();
        let mut program2 = programs.pop().unwrap();

        let h1 = kyron::spawn(async move {
            let _ = program1.run_n(3).await;
        });

        let h2 = kyron::spawn(async move {
            let _ = program2.run_n(3).await;
        });

        let _ = h1.await;
        let _ = h2.await;

        info!("Programs finished running");
    });
}
