use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

mod only_shutdown_sequence;

pub fn basic_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "basic",
        vec![Box::new(only_shutdown_sequence::OnlyShutdownSequence)],
        vec![],
    ))
}
