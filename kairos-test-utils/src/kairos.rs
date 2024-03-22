use crate::postgres::PostgresDB;

use backoff::future::retry;
use backoff::ExponentialBackoff;
use reqwest::Url;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies, RetryTransientMiddleware};
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::process::{Child, Command};
use std::str::FromStr;
use tokio::net::TcpStream;

use zk_prover::registry::api as registry_api;
use zk_prover::registry::client as registry_client;

async fn wait_for_port(address: &SocketAddr) -> Result<(), io::Error> {
    retry(ExponentialBackoff::default(), async || {
        Ok(TcpStream::connect(address).await.map(|_| ())?)
    })
    .await
}

pub struct Kairos {
    pub url: Url,
    process_handle: Child,

impl Storer {
    pub async fn run() -> Result<Storer, io::Error> {
        let postgres = PostgresDB::run("STORER_MIGRATIONS_DIR")?;
        let postgres_port = postgres.connection.port.to_string();
        let port = TcpListener::bind("127.0.0.1:0")?
            .local_addr()?
            .port()
            .to_string();
        let url = Url::parse(format!("http://127.0.0.1:{}", port).as_str()).unwrap();
        let mut args = vec![
            "--port",
            &port,
            "--pg-host",
            &postgres.connection.host,
            "--pg-port",
            &postgres_port,
            "--pg-user",
            &postgres.connection.username,
            "--pg-db",
            &postgres.connection.database,
        ];
        if let Some(registry) = storer_registry_url {
            args.extend(vec!["--storer-registry-url", registry.as_str()])
        }
        let process_handle = Command::new(env!("CARGO_BIN_EXE_kairos-server"))
            .args(args)
            .spawn()
            .expect("Failed to start storer.");

        wait_for_port(url.socket_addrs(|| Option::None).unwrap().first().unwrap())
            .await
            .unwrap();

        Ok(Storer {
            url,
            process_handle,
            postgres,
        })
    }
}

impl Drop for Storer {
    fn drop(&mut self) {
        let _ = self.process_handle.kill();
    }
}

