use crate::internals::runtime_helper::Runtime;
use async_runtime::io::{AsyncReadExt, AsyncWrite};
use async_runtime::net::TcpStream;
use async_runtime::spawn;
use core::net::SocketAddr;
use futures::task::noop_waker_ref;
use serde_json::Value;
use std::net::IpAddr;
use std::pin::Pin;
use std::str::{self, FromStr};
use std::task::{Context, Poll};
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;

fn parse_socket_addr(input: &Option<String>) -> SocketAddr {
    let input_string = input.as_ref().expect("Test input is expected");
    let input_content: Value = serde_json::from_str(input_string).expect("Failed to parse input string");

    // Parse connection configuration.
    let connection: Value = serde_json::from_value(input_content["connection"].clone()).expect("Failed to parse \"connection\" field");

    // Parse IP and port.
    let ip_str = connection["ip"].as_str().expect("Failed to parse \"ip\" field");
    let ip = IpAddr::from_str(ip_str).expect("\"ip\" content is not valid");
    let port = connection["port"].as_u64().expect("Failed to parse \"port\" field") as u16;

    SocketAddr::new(ip, port)
}

fn parse_message(input: &Option<String>) -> String {
    let input_string = input.as_ref().expect("Test input is expected");
    let input_content: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
    input_content["message"].as_str().expect("Failed to parse \"message\" field").to_string()
}

async fn write_and_read_task(mut stream: TcpStream, message: String) {
    // Addresses.
    let peer_addr = stream.peer_addr().expect("Failed to get peer address");
    let local_addr = stream.local_addr().expect("Failed to get local address");
    info!(peer_addr = format!("{peer_addr:?}"), local_addr = format!("{local_addr:?}"));

    // Write.
    {
        let mut write_buf = [0u8; 1024];
        let data = message.as_bytes();
        write_buf[0..data.len()].copy_from_slice(data);

        let mut pinned = Pin::new(&mut stream);
        let waker = noop_waker_ref();
        let mut ctx = Context::from_waker(waker);

        let mut written = 0;
        while written < write_buf.len() {
            match AsyncWrite::poll_write(pinned.as_mut(), &mut ctx, &write_buf[written..write_buf.len()]) {
                Poll::Ready(Ok(0)) => {
                    info!("Client closed connection during write");
                    break;
                }
                Poll::Ready(Ok(m)) => {
                    written += m;
                    info!("Written {m} bytes");
                }
                Poll::Ready(Err(e)) => {
                    info!("Write error: {e:?}");
                    break;
                }
                Poll::Pending => {
                    info!("Write would block, try again later");
                    continue;
                }
            }
        }
    }

    // Read.
    {
        let mut read_buf = [0u8; 1024];
        match stream.read(&mut read_buf).await {
            Ok(0) => {
                info!("Client closed connection");
            }
            Ok(n) => {
                info!("Read {n} bytes");
            }
            Err(e) => {
                info!("Read error: {e:?}");
            }
        };

        let message_read = String::from_utf8(read_buf.to_vec()).expect("Failed to convert string from bytes");
        let message_read_trim = message_read.trim_end_matches(char::from(0));
        info!(message_read = message_read_trim);
    }
}

struct Smoke;

impl Scenario for Smoke {
    fn name(&self) -> &str {
        "smoke"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();
        let socket_addr = parse_socket_addr(&input);
        let message = parse_message(&input);
        let _ = rt.block_on(async move {
            let stream = TcpStream::connect(socket_addr).await.expect("Failed to connect");
            let _ = spawn(write_and_read_task(stream, message)).await;
            Ok(0)
        });

        Ok(())
    }
}

pub fn tcp_stream_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new("tcp_stream", vec![Box::new(Smoke)], vec![]))
}
