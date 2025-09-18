use crate::tests::orchestration::orchestration_shutdown::ShutdownBeforeStart;
use orchestration_concurrency::{MultipleConcurrency, NestedConcurrency, SingleConcurrency};
use orchestration_sequence::{AwaitSequence, NestedSequence, SingleSequence};
use orchestration_sleep::SleepUnderLoad;
use orchestration_trigger_sync::{
    OneTriggerOneSyncTwoPrograms, OneTriggerTwoSyncsThreePrograms, TriggerAndSyncInNestedBranches, TriggerSyncOneAfterAnother,
};
use orchestration_user_error_catch::{
    CatchConcurrencyUserError, CatchDoubleMixedUserError, CatchDoubleRecoverableUserError, CatchNestedConcurrencyUserError,
    CatchNestedSequenceUserError, CatchSequenceUserError, DoubleCatchSequence,
};
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

use orchestration_double_handler_catch::{CatchDoubleDiffHandlerError, CatchDoubleSameHandlerError};

use async_runtime::futures::reusable_box_future::ReusableBoxFuturePool;
use orchestration::{common::tag::Tag, prelude::*};

use orchestration_shutdown::{GetAllShutdowns, OneProgramNotShut, SingleProgramSingleShutdown, TwoProgramsSingleShutdown, TwoProgramsTwoShutdowns};
use tracing::info;

pub mod orchestration_user_error_catch;
macro_rules! generic_test_func {
    ($name:expr) => {
        || generic_test_sync_func($name)
    };
}
#[macro_use]
mod orchestration_concurrency;
mod orchestration_double_handler_catch;
mod orchestration_if_else;
mod orchestration_sequence;
mod orchestration_shutdown;
mod orchestration_sleep;
mod orchestration_trigger_sync;

fn sequence_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "sequence",
        vec![Box::new(SingleSequence), Box::new(NestedSequence), Box::new(AwaitSequence)],
        vec![],
    ))
}

fn concurrency_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "concurrency",
        vec![Box::new(SingleConcurrency), Box::new(MultipleConcurrency), Box::new(NestedConcurrency)],
        vec![],
    ))
}

fn trigger_sync_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "trigger_sync",
        vec![
            Box::new(OneTriggerOneSyncTwoPrograms),
            Box::new(OneTriggerTwoSyncsThreePrograms),
            Box::new(TriggerAndSyncInNestedBranches),
            Box::new(TriggerSyncOneAfterAnother),
        ],
        vec![],
    ))
}

fn sleep_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("sleep", vec![Box::new(SleepUnderLoad)], vec![]))
}

fn shutdown_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "shutdown",
        vec![
            Box::new(SingleProgramSingleShutdown),
            Box::new(TwoProgramsSingleShutdown),
            Box::new(TwoProgramsTwoShutdowns),
            Box::new(GetAllShutdowns),
            Box::new(OneProgramNotShut),
            Box::new(ShutdownBeforeStart),
        ],
        vec![],
    ))
}

fn catch_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "catch",
        vec![
            Box::new(CatchSequenceUserError),
            Box::new(CatchNestedSequenceUserError),
            Box::new(CatchConcurrencyUserError),
            Box::new(CatchDoubleRecoverableUserError),
            Box::new(CatchDoubleMixedUserError),
            Box::new(CatchDoubleSameHandlerError),
            Box::new(CatchDoubleDiffHandlerError),
            Box::new(CatchNestedConcurrencyUserError),
            Box::new(DoubleCatchSequence),
        ],
        vec![],
    ))
}

fn ifelse_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "if_else",
        vec![
            Box::new(orchestration_if_else::BasicIfElse),
            Box::new(orchestration_if_else::NestedIfElse),
        ],
        vec![],
    ))
}

pub fn orchestration_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "orchestration",
        vec![],
        vec![
            sequence_scenario_group(),
            concurrency_scenario_group(),
            trigger_sync_scenario_group(),
            sleep_scenario_group(),
            shutdown_scenario_group(),
            catch_scenario_group(),
            ifelse_scenario_group(),
        ],
    ))
}

pub struct JustLogAction {
    base: ActionBaseMeta,
    name: String,
}

impl JustLogAction {
    fn new(name: impl Into<String>) -> Box<JustLogAction> {
        const DEFAULT_TAG: &str = "integration::tests::just_log_action";

        Box::new(Self {
            base: ActionBaseMeta {
                tag: Tag::from_str_static(DEFAULT_TAG),
                reusable_future_pool: ReusableBoxFuturePool::for_value(1, Self::execute_impl("JustLogAction".into())),
            },
            name: name.into(),
        })
    }
    async fn execute_impl(name: String) -> ActionResult {
        info!("{name} was executed");
        Ok(())
    }
}

impl ActionTrait for JustLogAction {
    fn name(&self) -> &'static str {
        "JustLogAction"
    }
    fn dbg_fmt(&self, _nest: usize, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
    fn try_execute(&mut self) -> ReusableBoxFutureResult {
        self.base.reusable_future_pool.next(JustLogAction::execute_impl(self.name.clone()))
    }
}

/// emulate some computing
fn busy_sleep() -> ActionResult {
    info!("Start sleeping");
    let mut ctr = 1_000_000;
    while ctr > 0 {
        ctr -= 1;
    }
    info!("End sleeping");
    Ok(())
}

fn generic_test_sync_func(name: &'static str) -> InvokeResult {
    info!("Start of '{}' function", name);
    // Spend some time to simulate work
    let _ = busy_sleep();
    info!("End of '{}' function", name);
    Ok(())
}
