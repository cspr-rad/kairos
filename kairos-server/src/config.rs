use casper_types::ContractHash;
use hex::FromHex;
use reqwest::Url;
use std::net::SocketAddr;
use std::time::Duration;
use std::{fmt, str::FromStr};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub socket_addr: SocketAddr,
    pub casper_rpc: Url,
    pub casper_sse: Url,
    pub kairos_demo_contract_hash: ContractHash,
    pub batch_config: BatchConfig,
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
                        contract_hash_string, err
                    )
                })
            })
            .map(ContractHash::new)?;
        let batch_config = BatchConfig::from_env()?;
        Ok(Self {
            socket_addr,
            casper_rpc,
            casper_sse,
            kairos_demo_contract_hash,
            batch_config,
        })
    }
}

/// Configuration for the trie state thread.
/// Currently only configures when a batch is committed and sent to the proving server.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Set by the environment variable `KAIROS_SERVER_MAX_BATCH_SIZE`.
    pub max_batch_size: Option<u64>,
    /// Set by the environment variable `KAIROS_SERVER_MAX_BATCH_SECONDS`.
    pub max_batch_duration: Option<Duration>,
    pub proving_server: Url,
}

impl BatchConfig {
    pub fn from_env() -> Result<Self, String> {
        let max_batch_size = parse_env_as_opt("KAIROS_SERVER_MAX_BATCH_SIZE")?;
        let max_batch_duration =
            parse_env_as_opt::<u64>("KAIROS_SERVER_MAX_BATCH_SECONDS")?.map(Duration::from_secs);
        let proving_server = parse_env_as::<Url>("KAIROS_PROVER_SERVER_URL")?;

        Ok(Self {
            max_batch_size,
            max_batch_duration,
            proving_server,
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

fn parse_env_as_opt<T>(env: &str) -> Result<Option<T>, String>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    match std::env::var(env) {
        Ok(val) => val
            .parse::<T>()
            .map(Some)
            .map_err(|e| format!("Failed to parse {}: {}", env, e)),
        Err(_) => Ok(None),
    }
}
