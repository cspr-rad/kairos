use backoff::future::retry;
use backoff::ExponentialBackoff;
use casper_types::ContractHash;
use reqwest::Url;
use std::io;
use std::net::{SocketAddr, TcpListener};
use tokio::net::TcpStream;

async fn wait_for_port(address: &SocketAddr) -> Result<(), io::Error> {
    retry(ExponentialBackoff::default(), || async {
        Ok(TcpStream::connect(address).await.map(|_| ())?)
    })
    .await
}

pub struct Kairos {
    pub url: Url,
    process_handle: tokio::task::JoinHandle<()>,
}

impl Kairos {
    pub async fn run(
        casper_rpc: Url,
        casper_sse: Url,
        kairos_demo_contract_hash: Option<ContractHash>,
    ) -> Result<Kairos, io::Error> {
        let socket_addr = TcpListener::bind("0.0.0.0:0")?.local_addr()?;
        let port = socket_addr.port().to_string();
        let url = Url::parse(&format!("http://0.0.0.0:{}", port)).unwrap();
        let config = kairos_server::config::ServerConfig {
            socket_addr,
            casper_rpc,
            casper_sse,
            kairos_demo_contract_hash: kairos_demo_contract_hash.unwrap_or_default(),
        };

        let process_handle = tokio::spawn(async move {
            tracing_subscriber::fmt::init();
            kairos_server::run(config).await;
        });

        wait_for_port(&socket_addr).await.unwrap();

        Ok(Kairos {
            url,
            process_handle,
        })
    }
}

impl Drop for Kairos {
    fn drop(&mut self) {
        self.process_handle.abort()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_kairos_starts_and_terminates() {
        let dummy_rpc = Url::parse("http://127.0.0.1:11101/rpc").unwrap();
        let dummy_sse = Url::parse("http://127.0.0.1:11101/events/main").unwrap();
        let _kairos = Kairos::run(dummy_rpc, dummy_sse, None).await.unwrap();
    }
}
