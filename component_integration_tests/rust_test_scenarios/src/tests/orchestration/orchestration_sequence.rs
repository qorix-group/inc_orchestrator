use crate::internals::helpers::runtime_helper::Runtime;
use crate::internals::scenario::Scenario;

use super::*;
use foundation::prelude::*;
use orchestration::{
    api::{Orchestration, design::Design},
    common::DesignConfig,
};
pub struct SingleSequence;

fn single_sequence_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SingleSequence".into(), DesignConfig::default());

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_body(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("Action1"))
                .with_step(JustLogAction::new("Action2"))
                .with_step(JustLogAction::new("Action3"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks three actions in a single sequence execution
impl Scenario for SingleSequence {
    fn get_name(&self) -> &'static str {
        "single_sequence"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(single_sequence_design().expect("Failed to create design"))
            .design_done();

        // Create programs
        let mut programs = orch.create_programs().unwrap();

        // Put programs into runtime and run them
        let _ = rt.block_on(async move {
            let _ = programs.programs.pop().unwrap().run_n(1).await;
            info!("Program finished running.");
            Ok(0)
        });

        Ok(())
    }
}

pub struct NestedSequence;

fn nested_sequence_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("NestedSequence".into(), DesignConfig::default());

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_body(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("OuterAction1"))
                .with_step(
                    SequenceBuilder::new()
                        .with_step(JustLogAction::new("InnerAction1"))
                        .with_step(JustLogAction::new("InnerAction2"))
                        .build(),
                )
                .with_step(JustLogAction::new("OuterAction2"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks actions in a inner and outer sequence execution
impl Scenario for NestedSequence {
    fn get_name(&self) -> &'static str {
        "nested_sequence"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(nested_sequence_design().expect("Failed to create design"))
            .design_done();

        // Create programs
        let mut programs = orch.create_programs().unwrap();

        // Put programs into runtime and run them
        let _ = rt.block_on(async move {
            let _ = programs.programs.pop().unwrap().run_n(1).await;
            info!("Program finished running.");
            Ok(0)
        });

        Ok(())
    }
}

pub struct AwaitSequence;

fn awaited_sequence_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("AwaitedSequence".into(), DesignConfig::default());

    let evt1 = design.register_event(Tag::from_str_static("Test_Event_1"))?;

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_body(
            SequenceBuilder::new()
                .with_step(JustLogAction::new("Action1"))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(
                            SequenceBuilder::new()
                                .with_step(JustLogAction::new("Action2"))
                                .with_step(JustLogAction::new("Action3"))
                                .build(),
                        )
                        .with_branch(
                            SequenceBuilder::new()
                                .with_step(JustLogAction::new("Action4"))
                                .with_step(SyncBuilder::from_tag(&evt1))
                                .with_step(TriggerBuilder::from_tag(&evt1))
                                .with_step(JustLogAction::new("Action5"))
                                .build(),
                        )
                        .build(),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks three actions in a single sequence execution
impl Scenario for AwaitSequence {
    fn get_name(&self) -> &'static str {
        "await_sequence"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(awaited_sequence_design().expect("Failed to create design"))
            .design_done();

        // Create programs
        let mut programs = orch.create_programs().unwrap();

        // Put programs into runtime and run them
        let _ = rt.block_on(async move {
            let _ = programs.programs.pop().unwrap().run_n(1).await;
            info!("Program finished running.");
            Ok(0)
        });

        Ok(())
    }
}
