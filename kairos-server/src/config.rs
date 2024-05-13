use std::net::SocketAddr;
use std::{fmt, str::FromStr};

use reqwest::Url;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub casper_rpc: Url,
    pub casper_contract_hash: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let socket_addr = parse_env_as::<SocketAddr>("KAIROS_SERVER_SOCKET_ADDR")?;
        let casper_rpc = parse_env_as::<Url>("CASPER_RPC")?;
        let casper_contract_hash = parse_env_as::<String>("CASPER_CONTRACT_HASH")?;
        Ok(Self {
            socket_addr,
            casper_rpc,
            casper_contract_hash,
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
