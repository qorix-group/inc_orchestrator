use crate::internals::net_helper::{create_udp_listener, ConnectionParameters};
use crate::internals::runtime_helper::Runtime;
use async_runtime::{net::UdpSocket, spawn};
use std::sync::Arc;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::{debug, info};

async fn receive_and_echo(udp_socket: Arc<UdpSocket>) {
    let mut buf = [0u8; 1024];
    match udp_socket.recv_from(&mut buf).await {
        Ok((n, sender_addr)) => {
            debug!("Read {} bytes from {}: {:?}", n, sender_addr, &buf[..n]);

            let mut written = 0;
            while written < n {
                match udp_socket.send_to(&buf[written..n], sender_addr).await {
                    Ok(m) => {
                        written += m;
                        debug!("Written {} bytes", m);
                    }
                    Err(e) => {
                        info!("Write error: {:?}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            info!("Read error: {:?}", e);
        }
    }
}

pub struct UdpServerEcho;

impl Scenario for UdpServerEcho {
    fn name(&self) -> &str {
        "send_receive_echo"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");

        rt.block_on(async move {
            let listener = Arc::new(create_udp_listener(connection_parameters).await);

            loop {
                let _ = spawn(receive_and_echo(listener.clone())).await;
                // Loop is expected to be terminated by SIGTERM from Python test.
            }
        });

        Ok(())
    }
}

pub struct UdpServerLogTTL;

impl Scenario for UdpServerLogTTL {
    fn name(&self) -> &str {
        "log_ttl"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();
        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");

        rt.block_on(async move {
            let listener = Arc::new(create_udp_listener(connection_parameters).await);
            let ttl = listener.ttl().expect("Failed to get TTL value");
            info!(ttl);
        });

        Ok(())
    }
}

pub fn udp_server_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "udp_server",
        vec![Box::new(UdpServerEcho), Box::new(UdpServerLogTTL)],
        vec![],
    ))
}
