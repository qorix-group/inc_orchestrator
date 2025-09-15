use async_runtime::net::TcpListener;
use serde_json::Value;

pub struct NetHelper {
    ip: String,
    port: u64,
}

impl NetHelper {
    /// Creates a new NetHelper from the "connection" field in the input JSON.
    pub fn new(input: &Option<String>) -> Self {
        let input_string = input.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        let connection_field = &v["connection"];
        let ip = connection_field["ip"].as_str().expect("Missing 'ip' in connection input").to_string();
        let port = connection_field["port"].as_u64().expect("Missing 'port' in connection input");

        Self { ip, port }
    }

    pub async fn create_tcp_listener(&self) -> TcpListener {
        let address = format!("{}:{}", self.ip, self.port);
        TcpListener::bind(address)
            .await
            .map_err(|e| e.to_string())
            .expect("Failed to bind TCP listener")
    }
}
