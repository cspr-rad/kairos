use eventsource_stream::Eventsource;
use futures::stream::TryStreamExt;
use reqwest::Client;

use crate::types::{ExecutionResult, SseData};

const CASPER_SSE_SERVER: &str = "https://events.mainnet.casperlabs.io";
const EVENT_CHANNEL: &str = "/events/main";

mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SSE endpoint.
    let url = String::from(CASPER_SSE_SERVER) + EVENT_CHANNEL;
    let client = Client::new();
    let mut response = client.get(url).send().await?.bytes_stream().eventsource();

    // Receive handshake with API version.
    let handshake_event = response.try_next().await?.ok_or("Stream exhausted.")?;
    let handshake_data: SseData = serde_json::from_str(&handshake_event.data)?;
    let api_version = match handshake_data {
        SseData::ApiVersion(v) => Ok(v),
        _ => Err("Invalid handshake event"),
    }?;
    println!("API version: {}", api_version);

    // Handle incoming events - look for successfuly processed deployments.
    while let Some(event) = response.try_next().await? {
        let data: SseData = serde_json::from_str(&event.data)?;
        match data {
            SseData::ApiVersion(_) => Err("Unexpected handshake received.")?,
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
    Err("Stream exhausted.")?
}
