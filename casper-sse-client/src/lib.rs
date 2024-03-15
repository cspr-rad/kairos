use eventsource_stream::Eventsource;
use futures::stream::TryStreamExt;
use reqwest::Client;

use crate::error::SseError;
use crate::types::{ExecutionResult, SseData};

mod error;
mod types;

const DEFAULT_SSE_SERVER: &str = "https://events.mainnet.casperlabs.io";
const DEFAULT_EVENT_CHANNEL: &str = "/events/main";

pub struct SseListener {
    url: String,
}

impl SseListener {
    pub fn new(url: &str) -> Self {
        SseListener {
            url: url.to_string(),
        }
    }

    pub async fn listen_to_sse(&self) -> Result<(), SseError> {
        // Connect to SSE endpoint.
        let client = Client::new();
        let mut response = client
            .get(&self.url)
            .send()
            .await?
            .bytes_stream()
            .eventsource();

        // Receive handshake with API version.
        let handshake_event = response
            .try_next()
            .await?
            .ok_or(SseError::StreamExhausted)?;
        let handshake_data: SseData = serde_json::from_str(&handshake_event.data)?;
        let _api_version = match handshake_data {
            SseData::ApiVersion(v) => Ok(v),
            _ => Err(SseError::InvalidHandshake),
        }?;

        // Handle incoming events - look for successfuly processed deployments.
        while let Some(event) = response.try_next().await? {
            let data: SseData = serde_json::from_str(&event.data)?;
            match data {
                SseData::ApiVersion(_) => Err(SseError::UnexpectedHandshake)?,
                SseData::Other(_) => {}
                SseData::DeployProcessed {
                    execution_result,
                    deploy_hash,
                    account,
                } => {
                    if let ExecutionResult::Success(_) = execution_result {
                        println!(
                            "Deployment successful: {} | Public key: {}",
                            deploy_hash, account
                        );
                    }
                }
            }
        }

        // Stream was exhausted.
        Err(SseError::StreamExhausted)?
    }
}

impl Default for SseListener {
    fn default() -> Self {
        let url = format!("{}{}", DEFAULT_SSE_SERVER, DEFAULT_EVENT_CHANNEL);
        Self { url }
    }
}
