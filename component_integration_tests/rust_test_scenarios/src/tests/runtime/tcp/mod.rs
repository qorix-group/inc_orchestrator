mod server;

use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn tcp_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("tcp", vec![Box::new(server::TcpServer)], vec![]))
}
