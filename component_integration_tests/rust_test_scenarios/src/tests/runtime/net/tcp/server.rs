use crate::internals::net_helper::{create_tcp_listener, ConnectionParameters};
use crate::internals::runtime_helper::Runtime;
use crate::tests::runtime::net::tcp::handle_connection_with_echo_response;
use async_runtime::spawn;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

pub struct TcpServer;

impl Scenario for TcpServer {
    fn name(&self) -> &str {
        "basic_server"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");

        rt.block_on(async move {
            info!("Program entered engine");

            let listener = create_tcp_listener(connection_parameters).await;
            info!("TCP server listening on {}", listener.local_addr().expect("Failed to get local address"));

            // Loop is expected to be terminated by SIGTERM from Python test.
            loop {
                let (stream, _addr) = listener
                    .accept()
                    .await
                    .map_err(|e| e.to_string())
                    .expect("Failed to accept TCP connection");
                spawn(handle_connection_with_echo_response(stream));
            }
        });

        Ok(())
    }
}
