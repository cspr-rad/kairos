use backoff::future::retry;
use backoff::ExponentialBackoff;
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
    pub async fn run(casper_rpc: Url) -> Result<Kairos, io::Error> {
        let socket_addr = TcpListener::bind("0.0.0.0:0")?.local_addr()?;
        let port = socket_addr.port().to_string();
        let url = Url::parse(&format!("http://0.0.0.0:{}", port)).unwrap();
        let casper_contract_hash =
            String::from("0000000000000000000000000000000000000000000000000000000000000000");
        let config = kairos_server::config::ServerConfig {
            socket_addr,
            casper_rpc,
            casper_contract_hash,
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
        let _kairos = Kairos::run(dummy_rpc).await.unwrap();
    }
}
