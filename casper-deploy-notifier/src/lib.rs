use eventsource_stream::{Event, EventStreamError, Eventsource};
use futures::stream::{BoxStream, TryStreamExt};
use tokio::sync::mpsc;

pub use crate::error::SseError;
use crate::sse_types::SseData;
pub use crate::types::Notification;

mod error;
mod sse_types;
mod types;

const DEFAULT_SSE_SERVER: &str = "https://events.mainnet.casperlabs.io";
const DEFAULT_EVENT_CHANNEL: &str = "/events/main";

type BoxedEventStream = BoxStream<'static, Result<Event, EventStreamError<reqwest::Error>>>;

pub struct DeployNotifier {
    url: String,
    event_stream: Option<BoxedEventStream>,
}

impl Default for DeployNotifier {
    fn default() -> Self {
        let url = format!("{}{}", DEFAULT_SSE_SERVER, DEFAULT_EVENT_CHANNEL);
        Self {
            url,
            event_stream: None,
        }
    }
}

impl DeployNotifier {
    pub fn new(url: &str) -> Self {
        DeployNotifier {
            url: url.to_string(),
            event_stream: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), SseError> {
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

        // Wrap stream with box and store it.
        let boxed_event_stream = Box::pin(event_stream);
        self.event_stream = Some(boxed_event_stream);

        Ok(())
    }

    // Handle incoming events - look for successfuly processed deployments.
    // Before running this function again, make sure you established new connection.
    pub async fn run(&mut self, tx: mpsc::Sender<Notification>) -> Result<(), SseError> {
        // Take stream out of state.
        let mut event_stream = match self.event_stream.take() {
            Some(s) => Ok(s),
            None => Err(SseError::NotConnected),
        }?;

        while let Some(event) = event_stream.try_next().await? {
            let data: SseData = serde_json::from_str(&event.data)?;
            match data {
                SseData::ApiVersion(_) => Err(SseError::UnexpectedHandshake)?,
                SseData::Other(_) => {}
                SseData::DeployProcessed(event_details) => {
                    let notification = event_details.into();
                    if let Err(_e) = tx.send(notification).await {
                        // Receiver probably dropeed.
                        break;
                    }
                }
                SseData::Shutdown => Err(SseError::NodeShutdown)?,
            }
        }

        // Stream was exhausted.
        Err(SseError::StreamExhausted)?
    }
}
