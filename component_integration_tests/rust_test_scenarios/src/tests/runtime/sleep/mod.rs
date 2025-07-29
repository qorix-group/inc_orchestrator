mod sleep_basic;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn sleep_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("sleep", vec![Box::new(sleep_basic::SleepBasic)], vec![]))
}
