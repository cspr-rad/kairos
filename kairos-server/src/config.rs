use config::{Config as Configuration, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{fs::File as FsFile, net::SocketAddr, path::Path};
use tracing::{subscriber::set_global_default, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn serialize_level_filter<S>(level: &Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let level_str = match *level {
        Level::ERROR => "error",
        Level::WARN => "warn",
        Level::INFO => "info",
        Level::DEBUG => "debug",
        Level::TRACE => "trace",
    };

    serializer.serialize_str(level_str)
}

fn deserialize_level_filter<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "error" => Ok(Level::ERROR),
        "warn" => Ok(Level::WARN),
        "info" => Ok(Level::INFO),
        "debug" => Ok(Level::DEBUG),
        "trace" => Ok(Level::TRACE),
        _ => Err(serde::de::Error::custom("unknown level")),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Server {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Log {
    #[serde(
        deserialize_with = "deserialize_level_filter",
        serialize_with = "serialize_level_filter"
    )]
    pub level: Level,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub server: Server,
    pub log: Log,
}

#[allow(clippy::new_without_default)]
impl Settings {
    pub fn new() -> Self {
        dotenv().ok();

        // Start building the configuration
        let mut builder = Configuration::builder();

        // Check if the kairos_config.toml file exists before adding it as a source
        let config_path = "kairos_config.toml";
        if Path::new(config_path).exists() {
            builder = builder.add_source(File::new(config_path, config::FileFormat::Toml));
        }

        // Add environment variables as a source
        builder = builder.add_source(Environment::with_prefix("KAIROS").separator("_"));
        match builder.build() {
            Ok(config) => match config.try_deserialize::<Self>() {
                Ok(settings) => settings,
                Err(e) => {
                    eprintln!("Failed to deserialize config: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Failed to build config: {}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn socket_address(&self) -> SocketAddr {
        format!("{}:{}", self.server.address, self.server.port)
            .parse()
            .expect("Invalid address")
    }

    pub fn initialize_logger(&self) {
        let stdout_log = fmt::layer().pretty();
        let level_filter = EnvFilter::new(self.log.level.to_string());
        let subscriber = tracing_subscriber::Registry::default()
            .with(stdout_log)
            .with(level_filter);

        let file_log = self.log.file.as_ref().map(|file_name| {
            let file = FsFile::create(file_name).expect("Failed to open log file.");
            fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_writer(file)
        });

        if let Some(file_layer) = file_log {
            set_global_default(subscriber.with(file_layer))
                .expect("Unable to set global logging configuration");
        } else {
            set_global_default(subscriber).expect("Unable to set global logging configuration");
        }
    }
}
