mod server;
mod tcp_listener;
mod tcp_stream;

use async_runtime::io::{AsyncReadExt, AsyncWriteExt};
use async_runtime::net::TcpStream;
use server::server_group;
use tcp_listener::tcp_listener_group;
use tcp_stream::tcp_stream_group;
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

pub fn tcp_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "tcp",
        vec![],
        vec![tcp_stream_group(), tcp_listener_group(), server_group()],
    ))
}

// Implementation with read and write using external traits
async fn handle_connection_with_echo_response(mut stream: TcpStream) {
    // Addresses.
    let peer_addr = stream.peer_addr().expect("Failed to get peer address");
    let local_addr = stream.local_addr().expect("Failed to get local address");
    info!(peer_addr = format!("{peer_addr:?}"), local_addr = format!("{local_addr:?}"));

    // Read.
    let mut buf = [0u8; 1024];

    match stream.read(&mut buf).await {
        Ok(0) => {
            info!("Client closed connection");
        }
        Ok(n) => {
            info!("Read {n} bytes");
        }
        Err(e) => {
            info!("Read error: {e:?}");
        }
    }

    let message_read = String::from_utf8(buf.to_vec()).expect("Failed to convert string from bytes");
    let message_read_trim = message_read.trim_end_matches(char::from(0));
    info!(message_read = message_read_trim);

    // Write back the same message
    let data = message_read_trim.as_bytes();

    match stream.write(data).await {
        Ok(0) => {
            info!("Client closed connection during write");
        }
        Ok(n) => {
            info!("Written {n} bytes");
        }
        Err(e) => {
            info!("Write error: {e:?}");
        }
    }
}
