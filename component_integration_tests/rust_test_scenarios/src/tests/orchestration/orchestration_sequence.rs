use crate::internals::helpers::runtime_helper::Runtime;
use crate::internals::test_case::TestCase;

use super::*;
use orchestration::{prelude::*, program::ProgramBuilder};
pub struct SingleSequenceTest;

/// Checks three actions in a single sequence execution
impl TestCase for SingleSequenceTest {
    fn get_name(&self) -> &'static str {
        "single_sequence"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let _ = rt.block_on(async move {
            let mut program = ProgramBuilder::new(file!())
                .with_body(
                    Sequence::new_with_id(NamedId::new_static("Sequence1"))
                        .with_step(JustLogAction::new("Action1"))
                        .with_step(JustLogAction::new("Action2"))
                        .with_step(JustLogAction::new("Action3")),
                )
                .with_shutdown_notification(Sync::new("Shutdown"))
                .build();

            program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct NestedSequenceTest;

/// Checks actions in a inner and outer sequence execution
impl TestCase for NestedSequenceTest {
    fn get_name(&self) -> &'static str {
        "nested_sequence"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let _ = rt.block_on(async move {
            let mut program = ProgramBuilder::new(file!())
                .with_body(
                    Sequence::new_with_id(NamedId::new_static("OuterSequence"))
                        .with_step(JustLogAction::new("OuterAction1"))
                        .with_step(
                            Sequence::new_with_id(NamedId::new_static("InnerSequence"))
                                .with_step(JustLogAction::new("InnerAction1"))
                                .with_step(JustLogAction::new("InnerAction2")),
                        )
                        .with_step(JustLogAction::new("OuterAction2")),
                )
                .with_shutdown_notification(Sync::new("Shutdown"))
                .build();

            program.run_n(1).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct AwaitSequenceTest;

/// Checks three actions in a single sequence execution
impl TestCase for AwaitSequenceTest {
    fn get_name(&self) -> &'static str {
        "await_sequence"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let _ = rt.block_on(async move {
            let event_name: &str = "Test_Event_1";

            let mut program = ProgramBuilder::new(file!())
                .with_body(
                    Sequence::new_with_id(NamedId::new_static("OuterSequence"))
                        .with_step(JustLogAction::new("Action1"))
                        .with_step(
                            Concurrency::new_with_id(NamedId::new_static("Concurrency1"))
                                .with_branch(
                                    Sequence::new_with_id(NamedId::new_static("InnerSequence1"))
                                        .with_step(JustLogAction::new("Action2"))
                                        .with_step(JustLogAction::new("Action3")),
                                )
                                .with_branch(
                                    Sequence::new_with_id(NamedId::new_static("InnerSequence2"))
                                        .with_step(JustLogAction::new("Action4"))
                                        .with_step(Sync::new(event_name))
                                        .with_step(Trigger::new(event_name))
                                        .with_step(JustLogAction::new("Action5")),
                                ),
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
