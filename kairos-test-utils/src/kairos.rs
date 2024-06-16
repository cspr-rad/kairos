use backoff::future::retry;
use backoff::ExponentialBackoff;
use reqwest::Url;
use std::io;
use std::net::{SocketAddr, TcpListener};
use tokio::net::TcpStream;

use kairos_server::config::ServerConfig;

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
    /// If `requires_proving_server` is true, the proving server at `KAIROS_PROVER_SERVER_URL` will be used.
    /// If no proving server is running, we will start the one at `KAIROS_PROVER_SERVER_BIN`.
    /// This logic requires only one instance of `Kairos` with `requires_proving_server` set to true may be run at a time.
    /// Keep this constraint in mind when writing tests.
    ///
    /// Allowing multiple proving servers would degrade proving performance beyond the point of being useful.
    pub async fn run(casper_rpc: Url, requires_proving_server: bool) -> Result<Kairos, io::Error> {
        let socket_addr = TcpListener::bind("0.0.0.0:0")?.local_addr()?;
        let port = socket_addr.port().to_string();
        let url = Url::parse(&format!("http://0.0.0.0:{}", port)).unwrap();

        let mut config = ServerConfig::from_env().unwrap();
        config.casper_rpc = casper_rpc;
        config.socket_addr = socket_addr;

        let kairos_prover_server = match requires_proving_server {
            true if reqwest::get(config.proving_server.clone()).await.is_err() => {
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
        let _kairos = Kairos::run(dummy_rpc, false).await.unwrap();
    }
}
