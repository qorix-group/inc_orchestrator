pub mod single_rt_multiple_exec_engine;

use crate::internals::scenario::{ScenarioGroup, ScenarioGroupImpl};
use single_rt_multiple_exec_engine::SingleRtMultipleExecEngine;

pub struct ExecutionEngineScenarioGroup {
    group: ScenarioGroupImpl,
}

impl ExecutionEngineScenarioGroup {
    pub fn new() -> Self {
        ExecutionEngineScenarioGroup {
            group: ScenarioGroupImpl::new("execution_engine"),
        }
    }
}

impl ScenarioGroup for ExecutionEngineScenarioGroup {
    fn get_group_impl(&mut self) -> &mut ScenarioGroupImpl {
        &mut self.group
    }

    fn init(&mut self) {
        self.group.add_scenario(Box::new(SingleRtMultipleExecEngine));
    }
}
