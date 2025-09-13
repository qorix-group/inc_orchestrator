use async_runtime::net::TcpListener;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct NetHelper {
    ip: String,
    port: u64,
}

impl NetHelper {
    /// Creates a new NetHelper from the "connection" field in the input JSON.
    pub fn new(input: &Option<String>) -> Self {
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        serde_json::from_value(v["connection"].clone()).expect("Failed to parse \"connection\" field")
    }

    pub async fn create_tcp_listener(&self) -> TcpListener {
        let address = format!("{}:{}", self.ip, self.port);
        TcpListener::bind(address)
            .await
            .map_err(|e| e.to_string())
            .expect("Failed to bind TCP listener")
    }
}
