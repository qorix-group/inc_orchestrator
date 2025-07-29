mod single_rt_multiple_exec_engine;

use single_rt_multiple_exec_engine::SingleRtMultipleExecEngine;
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn execution_engine_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "execution_engine",
        vec![Box::new(SingleRtMultipleExecEngine)],
        vec![],
    ))
}
