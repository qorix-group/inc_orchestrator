mod spmc_broadcast;
mod spsc;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

use crate::tests::runtime::channel::spmc_broadcast::spmc_broadcast_scenario_group;
use crate::tests::runtime::channel::spsc::spsc_scenario_group;

pub fn channel_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "channel",
        vec![],
        vec![spsc_scenario_group(), spmc_broadcast_scenario_group()],
    ))
}
