mod udp_client;
mod udp_server;

use udp_client::udp_client_group;
use udp_server::udp_server_group;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn udp_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("udp", vec![], vec![udp_client_group(), udp_server_group()]))
}
