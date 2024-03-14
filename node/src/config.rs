use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use log::LevelFilter;
use toml;

// Custom serializer function
fn serialize_level_filter<S>(level: &LevelFilter, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let level_str = match level {
        LevelFilter::Off => "off",
        LevelFilter::Error => "error",
        LevelFilter::Warn => "warn",
        LevelFilter::Info => "info",
        LevelFilter::Debug => "debug",
        LevelFilter::Trace => "trace",
    };

    serializer.serialize_str(level_str)
}

// Custom deserializer function
fn deserialize_level_filter<'de, D>(deserializer: D) -> Result<LevelFilter, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "off" => Ok(LevelFilter::Off),
        "error" => Ok(LevelFilter::Error),
        "warn" => Ok(LevelFilter::Warn),
        "info" => Ok(LevelFilter::Info),
        "debug" => Ok(LevelFilter::Debug),
        "trace" => Ok(LevelFilter::Trace),
        _ => Ok(LevelFilter::Info),
    } 
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct API {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DB {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Log {
    #[serde(deserialize_with = "deserialize_level_filter", serialize_with = "serialize_level_filter")]
    pub level: LevelFilter,
    pub file_output: String,
    pub stdout: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub api: API,
    pub db: DB,
    pub log: Log,
}

impl Config {
    pub fn socket_address(&self) -> SocketAddr {
        format!("{}:{}", self.api.address, self.api.port)
            .parse()
            .expect("Invalid address")
    }

    pub fn db_address(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            &self.db.username,
            &self.db.password,
            &self.db.address,
            &self.db.port,
            &self.db.database,
        )
    }

    pub fn read_config() -> Config {
        let config_file = "config.toml";
        if !std::path::Path::new(config_file).exists() {
            // Generate and save config file
            let default_config = Config {
                api: API {
                    address: "0.0.0.0".to_string(),
                    port: 8080,
                },
                db: DB {
                    address: "0.0.0.0".to_string(),
                    port: 5432,
                    username: "kairos".to_string(),
                    password: "changeme".to_string(),
                    database: "kairos".to_string(),
                },
                log: Log {
                    level: LevelFilter::Info,
                    file_output: "contradiction.log".to_string(),
                    stdout: true,
                },
            };
            let toml_string = toml::to_string_pretty(&default_config).unwrap();
            std::fs::write(config_file, toml_string).expect("Failed to write config file");
            eprintln!("Config file not found. Generated a default config file.");
        }

        // Read config from file
        let config: Config = toml::from_str(
            &std::fs::read_to_string(config_file).expect("Failed to read config file")
        ).expect("Failed to parse config file");

        config
    }
}