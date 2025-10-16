mod tcp;
mod udp;

use crate::tests::runtime::net::tcp::tcp_scenario_group;
use crate::tests::runtime::net::udp::udp_scenario_group;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn net_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("net", vec![], vec![tcp_scenario_group(), udp_scenario_group()]))
}
