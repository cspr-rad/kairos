use eventsource_stream::{Event, EventStreamError, Eventsource};
use futures::stream::{BoxStream, TryStreamExt};

use crate::error::SseError;
use crate::sse_types::{ExecutionResult, SseData};

mod error;
mod sse_types;

const DEFAULT_SSE_SERVER: &str = "https://events.mainnet.casperlabs.io";
const DEFAULT_EVENT_CHANNEL: &str = "/events/main";

type BoxedEventStream = BoxStream<'static, Result<Event, EventStreamError<reqwest::Error>>>;

pub struct DeployNotifier {
    url: String,
}

impl Default for DeployNotifier {
    fn default() -> Self {
        let url = format!("{}{}", DEFAULT_SSE_SERVER, DEFAULT_EVENT_CHANNEL);
        Self { url }
    }
}

impl DeployNotifier {
    pub fn new(url: &str) -> Self {
        DeployNotifier {
            url: url.to_string(),
        }
    }

    async fn connect(&self) -> Result<BoxedEventStream, SseError> {
        // Connect to SSE endpoint.
        let client = reqwest::Client::new();
        let response = client.get(&self.url).send().await?;

        let stream = response.bytes_stream();
        let mut event_stream = stream.eventsource();

        // Handle the handshake with API version.
        let handshake_event = event_stream
            .try_next()
            .await?
            .ok_or(SseError::StreamExhausted)?;
        let handshake_data: SseData = serde_json::from_str(&handshake_event.data)?;
        let _api_version = match handshake_data {
            SseData::ApiVersion(v) => Ok(v),
            _ => Err(SseError::InvalidHandshake),
        }?;

        // Wrap stream with box.
        let boxed_event_stream = Box::pin(event_stream);

        Ok(boxed_event_stream)
    }

    // Handle incoming events - look for successfuly processed deployments.
    pub async fn run(&mut self) -> Result<(), SseError> {
        let mut event_stream = self.connect().await?;

        while let Some(event) = event_stream.try_next().await? {
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
