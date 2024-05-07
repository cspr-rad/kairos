use std::net::SocketAddr;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub casper_node_url: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let socket_addr = parse_env_as::<SocketAddr>("KAIROS_SERVER_SOCKET_ADDR")?;
        let casper_node_url = parse_env_as::<String>("KAIROS_CASPER_NODE_URL")?;
        Ok(Self {
            socket_addr,
            casper_node_url,
        })
    }
}

fn parse_env_as<T>(env: &str) -> Result<T, String>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    std::env::var(env)
        .map_err(|e| format!("Failed to fetch environment variable {}: {}", env, e))
        .and_then(|val| {
            val.parse::<T>()
                .map_err(|e| format!("Failed to parse {}: {}", env, e))
        })
}
