use crate::internals::runtime_helper::Runtime;
use test_scenarios_rust::scenario::Scenario;

use super::*;
use foundation::prelude::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};

async fn dummy_task() -> InvokeResult {
    Ok(())
}

pub struct CatchDoubleSameHandlerError;

impl CatchDoubleSameHandlerError {
    fn create_design(&self) -> Result<Design, CommonErrors> {
        let mut design = Design::new("double_unrecoverable_catch_design".into(), DesignConfig::default());

        let dummy_tag = design.register_invoke_async("dummy_task".into(), dummy_task)?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    SequenceBuilder::new()
                        .with_step(
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&dummy_tag, design.config()))
                                .build(design),
                        )
                        .with_step(Invoke::from_tag(&dummy_tag, design.config()))
                        .build(),
                )
                .catch(|_e| ())
                .catch(|_e| ())
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchDoubleSameHandlerError {
    fn name(&self) -> &str {
        "double_same_handler_error"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let orch = Orchestration::new()
            .add_design(self.create_design().expect("Failed to create design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        let _ = rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct CatchDoubleDiffHandlerError;

impl CatchDoubleDiffHandlerError {
    fn create_design(&self) -> Result<Design, CommonErrors> {
        let mut design = Design::new("double_mixed_catch_design".into(), DesignConfig::default());

        let dummy_tag = design.register_invoke_async("dummy_task".into(), dummy_task)?;

        design.add_program("catch_program", move |design, builder| {
            builder.with_run_action(
                CatchBuilder::new(
                    ErrorFilter::UserErrors.into(),
                    SequenceBuilder::new()
                        .with_step(
                            ConcurrencyBuilder::new()
                                .with_branch(Invoke::from_tag(&dummy_tag, design.config()))
                                .build(design),
                        )
                        .with_step(Invoke::from_tag(&dummy_tag, design.config()))
                        .build(),
                )
                .catch(|_e| ())
                .catch_recoverable(|_e| false)
                .build(design),
            );

            Ok(())
        });

        Ok(design)
    }
}

impl Scenario for CatchDoubleDiffHandlerError {
    fn name(&self) -> &str {
        "double_diff_handler_error"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let orch = Orchestration::new()
            .add_design(self.create_design().expect("Failed to create design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        let _ = rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}
