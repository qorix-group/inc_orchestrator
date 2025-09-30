mod server;
mod tcp_listener;
mod tcp_stream;

use tcp_listener::tcp_listener_group;
use tcp_stream::tcp_stream_group;
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

pub fn tcp_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "tcp",
        vec![Box::new(server::TcpServer)],
        vec![tcp_stream_group(), tcp_listener_group()],
    ))
}
