use reqwest::Url;
use std::net::SocketAddr;
use std::{fmt, str::FromStr};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub casper_rpc: Url,
    pub db_addr: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let socket_addr = parse_env_as::<SocketAddr>("KAIROS_SERVER_SOCKET_ADDR")?;
        let casper_rpc = parse_env_as::<Url>("KAIROS_SERVER_CASPER_RPC")?;
        let db_addr = parse_env_as::<String>("KAIROS_SERVER_DB_ADDR")?;
        Ok(Self {
            socket_addr,
            casper_rpc,
            db_addr,
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
