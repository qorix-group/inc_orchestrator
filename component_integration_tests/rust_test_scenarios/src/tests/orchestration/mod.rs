use crate::internals::scenario::{ScenarioGroup, ScenarioGroupImpl};
use async_runtime::futures::{reusable_box_future::ReusableBoxFuturePool, yield_now};
use orchestration::common::tag::Tag;
use orchestration::prelude::*;
use orchestration_concurrency::{MultipleConcurrency, NestedConcurrency, SingleConcurrency};
use orchestration_sequence::{AwaitSequence, NestedSequence, SingleSequence};
use std::pin::Pin;
use tracing::info;

pub mod orchestration_concurrency;
pub mod orchestration_sequence;

pub struct OrchestrationScenarioGroup {
    group: ScenarioGroupImpl,
}

impl OrchestrationScenarioGroup {
    pub fn new() -> Self {
        OrchestrationScenarioGroup {
            group: ScenarioGroupImpl::new("orchestration"),
        }
    }
}

impl ScenarioGroup for OrchestrationScenarioGroup {
    fn get_group_impl(&mut self) -> &mut ScenarioGroupImpl {
        &mut self.group
    }

    fn init(&mut self) -> () {
        // Sequence scenarios
        self.group.add_scenario(Box::new(SingleSequence));
        self.group.add_scenario(Box::new(NestedSequence));
        self.group.add_scenario(Box::new(AwaitSequence));
        // Concurrency scenarios
        self.group.add_scenario(Box::new(SingleConcurrency));
        self.group.add_scenario(Box::new(MultipleConcurrency));
        self.group.add_scenario(Box::new(NestedConcurrency));
    }
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
                reusable_future_pool: ReusableBoxFuturePool::new(1, Self::execute_impl("JustLogAction".into())),
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

/// emulate some sleep as workaround until sleep is supported in runtime
#[allow(dead_code)]
fn busy_sleep() -> ActionResult {
    info!("Start sleeping");
    let mut ctr = 1_000_000;
    while ctr > 0 {
        ctr -= 1;
    }
    info!("End sleeping");
    Ok(())
}

#[allow(dead_code)]
async fn generic_test_func(name: &'static str) -> ActionResult {
    info!("Start of '{}' function", name);
    info!("'{}' function yielding...", name);
    yield_now::yield_now().await;
    info!("'{}' function resuming...", name);
    let rv = busy_sleep();
    info!("End of '{}' function", name);
    rv
}

#[allow(dead_code)]
fn factory_test_func(name: &'static str) -> impl Fn() -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + Clone {
    move || Box::pin(generic_test_func(name))
}

fn generic_test_sync_func(name: &'static str) -> InvokeResult {
    info!("Start of '{}' function", name);

    info!("End of '{}' function", name);
    Ok(())
}

pub fn function1() -> InvokeResult {
    generic_test_sync_func("Function1")
}
pub fn function2() -> InvokeResult {
    generic_test_sync_func("Function2")
}
pub fn function3() -> InvokeResult {
    generic_test_sync_func("Function3")
}
pub fn function4() -> InvokeResult {
    generic_test_sync_func("Function4")
}
pub fn function5() -> InvokeResult {
    generic_test_sync_func("Function5")
}
pub fn function6() -> InvokeResult {
    generic_test_sync_func("Function6")
}
pub fn outer_function1() -> InvokeResult {
    generic_test_sync_func("OuterFunction1")
}
pub fn outer_function2() -> InvokeResult {
    generic_test_sync_func("OuterFunction2")
}
pub fn inner_function1() -> InvokeResult {
    generic_test_sync_func("InnerFunction1")
}
pub fn inner_function2() -> InvokeResult {
    generic_test_sync_func("InnerFunction2")
}
