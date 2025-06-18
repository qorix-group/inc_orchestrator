use crate::internals::helpers::runtime_helper::Runtime;
use crate::internals::test_case::TestCase;

use super::*;
use orchestration::{prelude::*, program::ProgramBuilder};

pub struct SingleConcurrencyTest;

/// Checks Concurrency Functions
impl TestCase for SingleConcurrencyTest {
    fn get_name(&self) -> &'static str {
        "single_concurrency"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let _ = rt.block_on(async move {
            let mut program = ProgramBuilder::new(file!())
                .with_body(
                    Sequence::new_with_id(NamedId::new_static("Sequence"))
                        .with_step(
                            Concurrency::new_with_id(NamedId::new_static("Concurrency in Sequence"))
                                .with_branch(Invoke::from_async(factory_test_func("Function1")))
                                .with_branch(Invoke::from_async(factory_test_func("Function2")))
                                .with_branch(Invoke::from_async(factory_test_func("Function3"))),
                        )
                        .with_step(JustLogAction::new("FinishAction")),
                )
                .with_shutdown_notification(Sync::new("Shutdown"))
                .build();

            program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct MultipleConcurrencyTest;

/// Checks Concurrency Functions
impl TestCase for MultipleConcurrencyTest {
    fn get_name(&self) -> &'static str {
        "multiple_concurrency"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let _ = rt.block_on(async move {
            let mut program = ProgramBuilder::new(file!())
                .with_body(
                    Sequence::new_with_id(NamedId::new_static("Sequence"))
                        .with_step(
                            Concurrency::new_with_id(NamedId::new_static("Concurrency1 in Sequence"))
                                .with_branch(Invoke::from_async(factory_test_func("Function1")))
                                .with_branch(Invoke::from_async(factory_test_func("Function2")))
                                .with_branch(Invoke::from_async(factory_test_func("Function3"))),
                        )
                        .with_step(JustLogAction::new("IntermediateAction"))
                        .with_step(
                            Concurrency::new_with_id(NamedId::new_static("Concurrency2 in Sequence"))
                                .with_branch(Invoke::from_async(factory_test_func("Function4")))
                                .with_branch(Invoke::from_async(factory_test_func("Function5")))
                                .with_branch(Invoke::from_async(factory_test_func("Function6"))),
                        )
                        .with_step(JustLogAction::new("FinishAction")),
                )
                .with_shutdown_notification(Sync::new("Shutdown"))
                .build();

            program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct NestedConcurrencyTest;

/// Checks Concurrency Functions
impl TestCase for NestedConcurrencyTest {
    fn get_name(&self) -> &'static str {
        "nested_concurrency"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let _ = rt.block_on(async move {
            let mut program = ProgramBuilder::new(file!())
                .with_body(
                    Sequence::new_with_id(NamedId::new_static("Sequence"))
                        .with_step(
                            Concurrency::new_with_id(NamedId::new_static("Outer Concurrency in Sequence"))
                                .with_branch(Invoke::from_async(factory_test_func("OuterFunction1")))
                                .with_branch(
                                    Concurrency::new_with_id(NamedId::new_static("Inner Concurrency in Sequence"))
                                        .with_branch(Invoke::from_async(factory_test_func("InnerFunction1")))
                                        .with_branch(Invoke::from_async(factory_test_func("InnerFunction2"))),
                                )
                                .with_branch(Invoke::from_async(factory_test_func("OuterFunction2"))),
                        )
                        .with_step(JustLogAction::new("FinishAction")),
                )
                .with_shutdown_notification(Sync::new("Shutdown"))
                .build();

            program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}
