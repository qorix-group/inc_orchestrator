use crate::internals::net_helper::{create_tcp_listener, ConnectionParameters};
use crate::internals::runtime_helper::Runtime;
use crate::tests::runtime::net::tcp::handle_connection_with_echo_response;
use core::task::Poll;
use futures::future::poll_fn;
use kyron::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use kyron::net::TcpStream;
use kyron::spawn;
use std::io::Error;
use std::pin::Pin;
use test_scenarios_rust::scenario::{Scenario, ScenarioGroup, ScenarioGroupImpl};
use tracing::info;
pub struct TcpServer;

impl Scenario for TcpServer {
    fn name(&self) -> &str {
        "basic"
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

pub struct TcpNoResponseServer;
async fn handle_connection_no_response(mut stream: TcpStream) {
    // Addresses.
    let peer_addr = stream.peer_addr().expect("Failed to get peer address");
    let local_addr = stream.local_addr().expect("Failed to get local address");
    info!(peer_addr = format!("{peer_addr:?}"), local_addr = format!("{local_addr:?}"));

    // Read.
    let mut buf = [0u8; 1024];

    let n = match stream.read(&mut buf).await {
        Ok(0) => {
            info!("Client closed connection");
            0
        }
        Ok(n) => {
            info!("Read {n} bytes");
            n
        }
        Err(e) => {
            info!("Read error: {e:?}");
            0
        }
    };

    let message_read = String::from_utf8(buf[..n].to_vec()).expect("Failed to convert string from bytes");
    let message_read_trim = message_read.trim_end_matches(char::from(0));
    info!(message_read = message_read_trim);
}

impl Scenario for TcpNoResponseServer {
    fn name(&self) -> &str {
        "no_response"
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
                // No response handling
                spawn(handle_connection_no_response(stream));
            }
        });

        Ok(())
    }
}
pub struct TcpPollWriteServer;

pub async fn handle_connection_with_poll(mut stream: TcpStream) -> Result<(), Error> {
    // Addresses.
    let peer_addr = stream.peer_addr().expect("Failed to get peer address");
    let local_addr = stream.local_addr().expect("Failed to get local address");
    info!(peer_addr = format!("{peer_addr:?}"), local_addr = format!("{local_addr:?}"));

    // Read
    let mut buf = [0u8; 1024];
    let n = poll_fn(|cx| {
        let mut read_buf = ReadBuf::new(&mut buf);
        let mut pinned_stream = Pin::new(&mut stream);
        match pinned_stream.as_mut().poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(read_buf.filled().len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    })
    .await?;

    info!("Read {} bytes", n);

    // Decode message
    let message_read = String::from_utf8_lossy(&buf[..n]);
    let message_read_trim = message_read.trim_end_matches(char::from(0));
    info!(message_read = message_read_trim);

    // Write
    let data = message_read_trim.as_bytes();
    let mut total_written = 0;
    while total_written < data.len() {
        let written = poll_fn(|cx| {
            let mut pinned_stream = Pin::new(&mut stream);
            pinned_stream.as_mut().poll_write(cx, &data[total_written..])
        })
        .await?;
        if written == 0 {
            info!("Client closed connection during write");
            break;
        }
        total_written += written;
    }
    info!("Written {} bytes", total_written);

    Ok(())
}

impl Scenario for TcpPollWriteServer {
    fn name(&self) -> &str {
        "poll_read_write"
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

                // Call the poll-based handler
                spawn(handle_connection_with_poll(stream));
            }
        });

        Ok(())
    }
}

pub fn server_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "server",
        vec![Box::new(TcpServer), Box::new(TcpNoResponseServer), Box::new(TcpPollWriteServer)],
        vec![],
    ))
}
