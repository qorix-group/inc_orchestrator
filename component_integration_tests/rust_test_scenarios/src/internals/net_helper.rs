use async_runtime::net::{TcpListener, TcpStream};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::net::SocketAddr;

fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Fields {
        ip: String,
        port: u16,
    }

    let fields = Fields::deserialize(deserializer)?;
    let ip = fields.ip.parse().map_err(de::Error::custom)?;
    Ok(SocketAddr::new(ip, fields.port))
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConnectionParameters {
    #[serde(flatten, deserialize_with = "deserialize_socket_addr")]
    address: SocketAddr,
    ttl: Option<u32>,
}

impl ConnectionParameters {
    /// Parse `ConnectionParameters` from JSON string.
    /// JSON is expected to contain `connection` field.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let v: Value = serde_json::from_str(json_str)?;
        serde_json::from_value(v["connection"].clone())
    }

    /// Parse `ConnectionParameters` from `Value`.
    /// `Value` is expected to contain `connection` field.
    #[allow(unused)]
    pub fn from_value(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value["connection"].clone())
    }
}

pub async fn create_tcp_listener(connection_parameters: ConnectionParameters) -> TcpListener {
    let listener = TcpListener::bind(connection_parameters.address)
        .await
        .expect("Failed to bind TCP listener");

    // Set optional TTL.
    if let Some(ttl) = connection_parameters.ttl {
        listener.set_ttl(ttl).expect("Failed to set TTL value");
    }

    listener
}

pub async fn create_tcp_stream(connection_parameters: ConnectionParameters) -> TcpStream {
    let stream = TcpStream::connect(connection_parameters.address).await.expect("Failed to connect");

    // Set optional TTL.
    if let Some(ttl) = connection_parameters.ttl {
        stream.set_ttl(ttl).expect("Failed to set TTL value");
    }

    stream
}
