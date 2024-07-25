use casper_client_types::{ContractHash, SecretKey};
use hex::FromHex;
use reqwest::Url;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use std::{fmt, str::FromStr};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Set by the environment variable `KAIROS_SERVER_SECRET_KEY_FILE`.
    /// This is checked at startup to ensure SecretKey::from_file is successful.
    pub secret_key_file: Option<PathBuf>,
    pub socket_addr: SocketAddr,
    pub casper_rpc: Url,
    pub casper_sse: Url,
    pub casper_sync_interval: Duration,
    pub kairos_demo_contract_hash: ContractHash,
    pub batch_config: BatchConfig,
    #[cfg(feature = "database")]
    pub db_addr: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let socket_addr = parse_env_as::<SocketAddr>("KAIROS_SERVER_SOCKET_ADDR")?;

        let casper_rpc = parse_env_as::<Url>("KAIROS_SERVER_CASPER_RPC")?;
        let casper_sse = parse_env_as::<Url>("KAIROS_SERVER_CASPER_SSE")?;
        let casper_sync_interval =
            parse_env_as::<u64>("KAIROS_SERVER_CASPER_SYNC_INTERVAL").map(Duration::from_secs)?;

        if casper_sync_interval.as_secs() == 0 {
            return Err("Casper sync interval must be greater than 0".to_string());
        }

        let batch_config = BatchConfig::from_env()?;

        let secret_key_file =
            parse_env_as_opt::<String>("KAIROS_SERVER_SECRET_KEY_FILE")?.map(PathBuf::from);

        match &secret_key_file {
            Some(secret_key_file) => {
                if SecretKey::from_file(secret_key_file).is_err() {
                    return Err("Invalid secret key".to_string());
                }
            }
            None => {
                tracing::warn!("No secret key file provided. This server will not be able to sign batch deploys.");
            }
        }
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
        #[cfg(feature = "database")]
        let db_addr = parse_env_as::<String>("KAIROS_SERVER_DB_ADDR")?;

        Ok(Self {
            secret_key_file,
            socket_addr,
            casper_rpc,
            casper_sse,
            casper_sync_interval,
            kairos_demo_contract_hash,
            batch_config,
            #[cfg(feature = "database")]
            db_addr,
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
