use std::pin::Pin;

use async_runtime::{core::types::box_future, futures::yield_now};
use orchestration::prelude::{ActionResult, ActionTrait};
use tracing::info;

pub mod orchestration_concurrency;
pub mod orchestration_sequence;
pub mod orchestration_test_group;

pub struct JustLogAction {
    name: String,
}

impl JustLogAction {
    fn new(name: impl Into<String>) -> Box<JustLogAction> {
        Box::new(Self { name: name.into() })
    }
}

impl ActionTrait for JustLogAction {
    fn execute(&mut self) -> orchestration::actions::action::ActionFuture {
        let name = self.name.clone();
        box_future(async move {
            info!("{name} was executed");
            Ok(())
        })
    }
    fn name(&self) -> &'static str {
        "JustLogAction"
    }
    fn dbg_fmt(&self, _nest: usize, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
    fn fill_runtime_info(&mut self, _p: &mut orchestration::actions::action::ActionRuntimeInfoProvider) {}
}

/// emulate some sleep as workaround until sleep is supported in runtime
fn busy_sleep() -> ActionResult {
    info!("Start sleeping");
    let mut ctr = 1_000_000;
    while ctr > 0 {
        ctr -= 1;
    }
    info!("End sleeping");
    Ok(())
}

async fn generic_test_func(name: &'static str) -> ActionResult {
    info!("Start of '{}' function", name);
    info!("'{}' function yielding...", name);
    yield_now::yield_now().await;
    info!("'{}' function resuming...", name);
    let rv = busy_sleep();
    info!("End of '{}' function", name);
    rv
}

fn factory_test_func(name: &'static str) -> impl Fn() -> Pin<Box<dyn Future<Output = ActionResult> + Send>> + Clone {
    move || Box::pin(generic_test_func(name))
}
