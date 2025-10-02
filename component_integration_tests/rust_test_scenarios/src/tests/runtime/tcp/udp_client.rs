use crate::internals::net_helper::{create_default_udp_client, ConnectionParameters};
use crate::internals::runtime_helper::Runtime;
use async_runtime::{net::UdpSocket, spawn};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::{debug, info};

fn parse_message(input: &str) -> String {
    let input_content: Value = serde_json::from_str(input).expect("Failed to parse input string");
    input_content["message"].as_str().expect("Failed to parse \"message\" field").to_string()
}

fn parse_client_type(input: &str) -> String {
    let input_content: Value = serde_json::from_str(input).expect("Failed to parse input string");
    input_content["client_type"]
        .as_str()
        .expect("Failed to parse \"client_type\" field")
        .to_string()
}

async fn send_receive_log(udp_socket: Arc<UdpSocket>, address: SocketAddr, message: String) {
    let mut buf = [0u8; 1024];
    let msg_len = message.len();

    buf[..msg_len].copy_from_slice(message.as_bytes());
    debug!("Message to send: {message}");

    let mut written = 0;
    while written < msg_len {
        match udp_socket.send_to(&buf[written..msg_len], address).await {
            Ok(m) => {
                written += m;
                debug!("Written {} bytes", m);
            }
            Err(e) => {
                info!(send_to_error = format!("Write error: {:?}", e));
                break;
            }
        }
    }

    debug!("Waiting for the response...");
    match udp_socket.recv_from(&mut buf).await {
        Ok((n, sender_addr)) => {
            let received_message = String::from_utf8_lossy(&buf[..n]).into_owned();
            let address = format!("{}:{}", sender_addr.ip(), sender_addr.port());
            info!(received_bytes = n, received_message, address);
        }
        Err(e) => {
            info!(recv_from_error = format!("Read error: {:?}", e));
        }
    }
}

async fn connect_send_receive_log(udp_socket: Arc<UdpSocket>, address: SocketAddr, message: String) {
    let mut buf = [0u8; 1024];
    let msg_len = message.len();

    buf[..msg_len].copy_from_slice(message.as_bytes());
    debug!("Message to send: {message}");

    udp_socket.connect(address).await.expect("Failed to connect UDP socket");

    let mut written = 0;
    while written < msg_len {
        match udp_socket.send(&buf[written..msg_len]).await {
            Ok(m) => {
                written += m;
                debug!("Written {} bytes", m);
            }
            Err(e) => {
                info!(send_error = format!("Write error: {:?}", e));
                break;
            }
        }
    }

    debug!("Waiting for the response...");
    match udp_socket.recv(&mut buf).await {
        Ok(n) => {
            let received_message = String::from_utf8_lossy(&buf[..n]).into_owned();
            info!(received_bytes = n, received_message);
        }
        Err(e) => {
            info!(recv_error = format!("Read error: {:?}", e));
        }
    }
}

pub struct UdpClientLogResponse;

impl Scenario for UdpClientLogResponse {
    fn name(&self) -> &str {
        "log_response"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");
        let message = parse_message(input);
        let client_type = parse_client_type(input);

        rt.block_on(async move {
            let client = Arc::new(create_default_udp_client(connection_parameters.get_address()).await);
            if client_type == "connected" {
                let _ = spawn(connect_send_receive_log(client.clone(), connection_parameters.get_address(), message)).await;
            } else if client_type == "unconnected" {
                let _ = spawn(send_receive_log(client.clone(), connection_parameters.get_address(), message)).await;
            } else {
                panic!("Client type must be either \"connected\" or \"unconnected\"");
            }
        });

        Ok(())
    }
}

async fn unconnected_send(udp_socket: Arc<UdpSocket>, message: String) {
    let mut buf = [0u8; 1024];
    let msg_len = message.len();

    buf[..msg_len].copy_from_slice(message.as_bytes());
    debug!("Message to send: {message}");

    let mut written = 0;
    while written < msg_len {
        match udp_socket.send(&buf[written..msg_len]).await {
            Ok(m) => {
                written += m;
                debug!("Written {} bytes", m);
            }
            Err(e) => {
                info!(send_error = format!("Write error: {:?}", e));
                break;
            }
        }
    }
}

pub struct UdpClientUnconnectedSendError;

impl Scenario for UdpClientUnconnectedSendError {
    fn name(&self) -> &str {
        "unconnected_send_error"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let mut rt = Runtime::from_json(input)?.build();

        let connection_parameters = ConnectionParameters::from_json(input).expect("Failed to parse connection parameters");
        let message = parse_message(input);
        let client_type = parse_client_type(input);

        rt.block_on(async move {
            let client = Arc::new(create_default_udp_client(connection_parameters.get_address()).await);
            if client_type == "unconnected" {
                let _ = spawn(unconnected_send(client.clone(), message)).await;
            } else {
                panic!("Client type must be \"unconnected\"");
            }
        });

        Ok(())
    }
}

pub fn udp_client_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "udp_client",
        vec![Box::new(UdpClientLogResponse), Box::new(UdpClientUnconnectedSendError)],
        vec![],
    ))
}
