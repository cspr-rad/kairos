use backoff::future::retry;
use backoff::ExponentialBackoff;
use casper_client_types::ContractHash;
use reqwest::Url;
use std::io;
use std::net::{SocketAddr, TcpListener};
use tokio::net::TcpStream;

use kairos_server::config::{BatchConfig, ServerConfig};

async fn wait_for_port(address: &SocketAddr) -> Result<(), io::Error> {
    retry(ExponentialBackoff::default(), || async {
        Ok(TcpStream::connect(address).await.map(|_| ())?)
    })
    .await
}

pub struct Kairos {
    pub url: Url,
    task_handle: tokio::task::JoinHandle<()>,
    kairos_prover_server: Option<std::process::Child>,
}

impl Kairos {
    /// If no proving server is running, we will start the one at `BatchConfig.proving_server`.
    /// The caller should ensure that `BatchConfig.proving_server == KAIROS_PROVER_SERVER_URL`.
    pub async fn run(
        casper_rpc: &Url,
        casper_sse: &Url,
        proving_server_batch_config: Option<BatchConfig>,
        kairos_demo_contract_hash: Option<ContractHash>,
    ) -> Result<Kairos, io::Error> {
        let socket_addr = TcpListener::bind("0.0.0.0:0")?.local_addr()?;
        let port = socket_addr.port().to_string();
        let url = Url::parse(&format!("http://0.0.0.0:{}", port)).unwrap();
        #[cfg(feature = "database")]
        let db_addr = "postgres://kairos:kairos@localhost/kairos".to_string();

        let batch_config = proving_server_batch_config
            .clone()
            .unwrap_or_else(|| BatchConfig {
                max_batch_size: None,
                max_batch_duration: None,
                proving_server: Url::parse("http://127.0.0.1:7894").unwrap(),
            });

        let config = ServerConfig {
            secret_key_file: None,
            socket_addr,
            casper_rpc: casper_rpc.clone(),
            casper_sse: casper_sse.clone(),
            kairos_demo_contract_hash: kairos_demo_contract_hash.unwrap_or_default(),
            batch_config,
            #[cfg(feature = "database")]
            db_addr,
        };

        let kairos_prover_server = match proving_server_batch_config {
            Some(batch_config)
                if reqwest::get(batch_config.proving_server.clone())
                    .await
                    .is_err() =>
            {
                // Start the proving server if it's not providing any response.
                // We don't care what the response is, we just want to know it's reachable.
                let proving_server_bin = std::env::var("KAIROS_PROVER_SERVER_BIN").unwrap();

                match std::env::var("RISC0_DEV_MODE").map(|s| s == "1") {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        tracing::warn!(
                            "RISE0_DEV_MODE is not set to 1.\n\
                            Proving will take a long time\n\
                            To enable proving acceleration you should ensure `{proving_server_bin}`\
                            has been compiled with either --features=metal or --features=cuda.\n\
                            You can also run the proving server with acceleration in your terminal\n\
                            $ cd kairos-prover\n\
                            $ nix develop .#risczero\n\
                            $ cargo run --features=metal or cuda\n",
                        );
                    }
                }

                tracing::info!("Starting proving server at {}", proving_server_bin);
                // using std instead of tokio because we don't care about blocking tokio here.
                Some(
                    std::process::Command::new(proving_server_bin)
                        .spawn()
                        .expect("Failed to start proving server"),
                )
            }
            // We don't need a proving server, or it's already running.
            _ => None,
        };

        let process_handle = tokio::spawn(async move {
            tracing_subscriber::fmt::init();
            kairos_server::run(config).await;
        });

        wait_for_port(&socket_addr).await.unwrap();

        Ok(Kairos {
            url,
            task_handle: process_handle,
            kairos_prover_server,
        })
    }
}

impl Drop for Kairos {
    fn drop(&mut self) {
        self.task_handle.abort();
        if let Some(child) = self.kairos_prover_server.as_mut() {
            child.kill().expect("Failed to kill proving server");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_kairos_starts_and_terminates() {
        let dummy_rpc = Url::parse("http://127.0.0.1:11101/rpc").unwrap();
        let dummy_sse = Url::parse("http://127.0.0.1:18101/events/main").unwrap();
        let _kairos = Kairos::run(&dummy_rpc, &dummy_sse, None, None)
            .await
            .unwrap();
    }
}
