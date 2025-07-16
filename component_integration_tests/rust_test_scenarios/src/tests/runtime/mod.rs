pub mod execution_engine;
pub mod sleep;
pub mod worker;

use crate::internals::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub struct RuntimeScenarioGroup {
    group: ScenarioGroupImpl,
}

impl RuntimeScenarioGroup {
    pub fn new() -> Self {
        RuntimeScenarioGroup {
            group: ScenarioGroupImpl::new("runtime"),
        }
    }
}

impl ScenarioGroup for RuntimeScenarioGroup {
    fn get_group_impl(&mut self) -> &mut ScenarioGroupImpl {
        &mut self.group
    }

    fn init(&mut self) -> () {
        self.group.add_group(Box::new(execution_engine::ExecutionEngineScenarioGroup::new()));
        self.group.add_group(Box::new(sleep::SleepScenarioGroup::new()));
        self.group.add_group(Box::new(worker::WorkerScenarioGroup::new()));
    }
}
