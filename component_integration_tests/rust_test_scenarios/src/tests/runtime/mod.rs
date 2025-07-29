mod execution_engine;
mod sleep;
mod worker;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

use crate::tests::runtime::execution_engine::execution_engine_scenario_group;
use crate::tests::runtime::sleep::sleep_scenario_group;
use crate::tests::runtime::worker::worker_scenario_group;

pub fn runtime_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "runtime",
        vec![],
        vec![execution_engine_scenario_group(), sleep_scenario_group(), worker_scenario_group()],
    ))
}
