use crate::internals::net_helper::{create_tcp_listener, ConnectionParameters};
use crate::internals::runtime_helper::Runtime;
use crate::tests::runtime::tcp::handle_connection_with_echo_response;
use async_runtime::net::TcpListener;
use async_runtime::spawn;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

struct Smoke;

impl Scenario for Smoke {
    fn name(&self) -> &str {
        "smoke"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");

        let _ = rt.block_on(async move {
            let listener = create_tcp_listener(connection_parameters).await;
            info!("TCP server listening on {}", listener.local_addr().expect("Failed to get local address"));

            // Loop is expected to be terminated by SIGTERM from Python test.
            loop {
                let (stream, _addr) = listener.accept().await.expect("Failed to accept TCP connection");
                spawn(handle_connection_with_echo_response(stream));
            }
        });

        Ok(())
    }
}

async fn print_listener_ttl(listener: TcpListener) {
    let ttl = listener.ttl().expect("Failed to get TTL value");
    info!(ttl);
}

struct SetGetTtl;

impl Scenario for SetGetTtl {
    fn name(&self) -> &str {
        "set_get_ttl"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");

        let _ = rt.block_on(async move {
            let listener = create_tcp_listener(connection_parameters).await;

            let _ = spawn(print_listener_ttl(listener)).await;
            Ok(0)
        });

        Ok(())
    }
}

pub fn tcp_listener_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("tcp_listener", vec![Box::new(Smoke), Box::new(SetGetTtl)], vec![]))
}
