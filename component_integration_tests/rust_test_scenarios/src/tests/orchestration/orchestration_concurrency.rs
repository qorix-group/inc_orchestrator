use crate::internals::helpers::runtime_helper::Runtime;
use crate::internals::scenario::Scenario;

use super::*;
use foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};

pub struct SingleConcurrency;

fn single_concurrency_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SingleConcurrency".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("Function1".into(), generic_test_func!("Function1"))?;
    let t2_tag = design.register_invoke_fn("Function2".into(), generic_test_func!("Function2"))?;
    let t3_tag = design.register_invoke_fn("Function3".into(), generic_test_func!("Function3"))?;

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t1_tag))
                        .with_branch(Invoke::from_tag(&t2_tag))
                        .with_branch(Invoke::from_tag(&t3_tag))
                        .build(),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks Concurrency Functions
impl Scenario for SingleConcurrency {
    fn get_name(&self) -> &'static str {
        "single_concurrency"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(single_concurrency_design().expect("Failed to create design"))
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

pub struct MultipleConcurrency;

fn multiple_concurrency_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("MultipleConcurrency".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("Function1".into(), generic_test_func!("Function1"))?;
    let t2_tag = design.register_invoke_fn("Function2".into(), generic_test_func!("Function2"))?;
    let t3_tag = design.register_invoke_fn("Function3".into(), generic_test_func!("Function3"))?;
    let t4_tag = design.register_invoke_fn("Function4".into(), generic_test_func!("Function4"))?;
    let t5_tag = design.register_invoke_fn("Function5".into(), generic_test_func!("Function5"))?;
    let t6_tag = design.register_invoke_fn("Function6".into(), generic_test_func!("Function6"))?;
    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t1_tag))
                        .with_branch(Invoke::from_tag(&t2_tag))
                        .with_branch(Invoke::from_tag(&t3_tag))
                        .build(),
                )
                .with_step(JustLogAction::new("IntermediateAction"))
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t4_tag))
                        .with_branch(Invoke::from_tag(&t5_tag))
                        .with_branch(Invoke::from_tag(&t6_tag))
                        .build(),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks Concurrency Functions
impl Scenario for MultipleConcurrency {
    fn get_name(&self) -> &'static str {
        "multiple_concurrency"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(multiple_concurrency_design().expect("Failed to create design"))
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

pub struct NestedConcurrency;

fn nested_concurrency_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("NestedConcurrency".into(), DesignConfig::default());

    let t1_tag = design.register_invoke_fn("OuterFunction1".into(), generic_test_func!("OuterFunction1"))?;
    let t2_tag = design.register_invoke_fn("InnerFunction1".into(), generic_test_func!("InnerFunction1"))?;
    let t3_tag = design.register_invoke_fn("InnerFunction2".into(), generic_test_func!("InnerFunction2"))?;
    let t4_tag = design.register_invoke_fn("OuterFunction2".into(), generic_test_func!("OuterFunction2"))?;

    // Create a program with actions
    design.add_program(file!(), move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(
                    ConcurrencyBuilder::new()
                        .with_branch(Invoke::from_tag(&t1_tag))
                        .with_branch(
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&t2_tag))
                                .with_branch(Invoke::from_tag(&t3_tag))
                                .build(),
                        )
                        .with_branch(Invoke::from_tag(&t4_tag))
                        .build(),
                )
                .with_step(JustLogAction::new("FinishAction"))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks Concurrency Functions
impl Scenario for NestedConcurrency {
    fn get_name(&self) -> &'static str {
        "nested_concurrency"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        // Build Orchestration
        let orch = Orchestration::new()
            .add_design(nested_concurrency_design().expect("Failed to create design"))
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
