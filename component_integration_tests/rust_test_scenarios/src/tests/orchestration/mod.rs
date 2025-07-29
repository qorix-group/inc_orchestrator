use crate::tests::orchestration::orchestration_shutdown::ShutdownBeforeStart;
use orchestration_concurrency::{MultipleConcurrency, NestedConcurrency, SingleConcurrency};
use orchestration_sequence::{AwaitSequence, NestedSequence, SingleSequence};
use orchestration_sleep::SleepUnderLoad;
use orchestration_trigger_sync::{
    OneTriggerOneSyncTwoPrograms, OneTriggerTwoSyncsThreePrograms, TriggerAndSyncInNestedBranches, TriggerSyncOneAfterAnother,
};
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

use async_runtime::futures::reusable_box_future::ReusableBoxFuturePool;
use orchestration::{common::tag::Tag, prelude::*};

use orchestration_shutdown::{GetAllShutdowns, OneProgramNotShut, SingleProgramSingleShutdown, TwoProgramsSingleShutdown, TwoProgramsTwoShutdowns};
use tracing::info;

macro_rules! generic_test_func {
    ($name:expr) => {
        || generic_test_sync_func($name)
    };
}
#[macro_use]
mod orchestration_concurrency;
mod orchestration_sequence;
mod orchestration_shutdown;
mod orchestration_sleep;
mod orchestration_trigger_sync;

pub fn orchestration_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "orchestration",
        vec![
            // Sequence scenarios
            Box::new(SingleSequence),
            Box::new(NestedSequence),
            Box::new(AwaitSequence),
            // Concurrency scenarios
            Box::new(SingleConcurrency),
            Box::new(MultipleConcurrency),
            Box::new(NestedConcurrency),
            // Trigger and sync scenarios
            Box::new(OneTriggerOneSyncTwoPrograms),
            Box::new(OneTriggerTwoSyncsThreePrograms),
            Box::new(TriggerAndSyncInNestedBranches),
            Box::new(TriggerSyncOneAfterAnother),
            // Sleep scenarios
            Box::new(SleepUnderLoad),
            // Shutdown scenarios
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
