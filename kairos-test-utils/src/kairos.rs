use backoff::future::retry;
use backoff::ExponentialBackoff;
use reqwest::Url;
use std::env;
use std::io;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Command};
use tokio::net::TcpStream;

// A hacky way to get the cargo binary directory path
pub fn bin_dir() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // pop kairos_test_utils-hash
    path.pop(); // pop deps
    path
}

async fn wait_for_port(address: &SocketAddr) -> Result<(), io::Error> {
    retry(ExponentialBackoff::default(), || async {
        Ok(TcpStream::connect(address).await.map(|_| ())?)
    })
    .await
}

pub struct Kairos {
    pub url: Url,
    process_handle: Child,
}

impl Kairos {
    pub async fn run() -> Result<Kairos, io::Error> {
        let port = TcpListener::bind("127.0.0.1:0")?
            .local_addr()?
            .port()
            .to_string();
        let url = Url::parse(format!("http://127.0.0.1:{}", port).as_str()).unwrap();
        let kairos = bin_dir().join("kairos-server");
        let process_handle = Command::new(kairos)
            .env("KAIROS_SERVER_PORT", &port)
            .spawn()
            .expect("Failed to start the kairos-server");

        wait_for_port(url.socket_addrs(|| Option::None).unwrap().first().unwrap())
            .await
            .unwrap();

        Ok(Kairos {
            url,
            process_handle,
        })
    }
}

impl Drop for Kairos {
    fn drop(&mut self) {
        let _ = self.process_handle.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_kairos_starts_and_terminates() {
        let _kairos = Kairos::run().await.unwrap();
    }
}
