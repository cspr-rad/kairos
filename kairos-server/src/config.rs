use std::{fmt, str::FromStr};

#[derive(Debug)]
pub struct ServerConfig {
    pub port: u16,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let port = parse_env_as::<u16>("KAIROS_SERVER_PORT")?;
        Ok(Self { port })
    }
}

fn parse_env_as<T>(env: &str) -> Result<T, String>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    std::env::var(env)
        .map_err(|e| format!("Failed to parse {}: {}", env, e))
        .and_then(|val| {
            val.parse::<T>()
                .map_err(|e| format!("Failed to parse {}: {}", env, e))
        })
}
