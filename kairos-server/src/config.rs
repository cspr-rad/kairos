use casper_types::ContractHash;
use hex::FromHex;
use reqwest::Url;
use std::net::SocketAddr;
use std::{fmt, str::FromStr};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub casper_rpc: Url,
    pub casper_sse: Url,
    pub kairos_demo_contract_hash: ContractHash,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let socket_addr = parse_env_as::<SocketAddr>("KAIROS_SERVER_SOCKET_ADDR")?;
        let casper_rpc = parse_env_as::<Url>("KAIROS_SERVER_CASPER_RPC")?;
        let casper_sse = parse_env_as::<Url>("KAIROS_SERVER_CASPER_SSE")?;
        let kairos_demo_contract_hash = parse_env_as::<String>("KAIROS_SERVER_DEMO_CONTRACT_HASH")
            .and_then(|contract_hash_string| {
                <[u8; 32]>::from_hex(&contract_hash_string).map_err(|err| {
                    panic!(
                        "Failed to decode kairos-demo-contract-hash {}: {}",
                        contract_hash_string,
                        err
                    )
                })
            })
            .map(ContractHash::new)?;
        Ok(Self {
            socket_addr,
            casper_rpc,
            casper_sse,
            kairos_demo_contract_hash,
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
