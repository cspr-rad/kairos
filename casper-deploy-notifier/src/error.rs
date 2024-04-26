use eventsource_stream::EventStreamError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SseError {
    /// Unable to initiate connection with SSE endpoint.
    #[error("Failed to connect to SSE endpoint: {0}")]
    ConnectionError(#[from] reqwest::Error),

    /// Unable to run notifier, because there is no available connection.
    #[error("Not connected to event stream")]
    NotConnected,

    /// Connection issue with already opened stream.
    #[error("Stream error: {0}")]
    StreamError(#[from] EventStreamError<reqwest::Error>),

    /// Stream was gracefully ended - no more data to read.
    #[error("Stream exhausted")]
    StreamExhausted,

    /// Recevied invalid handshake event.
    #[error("Invalid handshake event")]
    InvalidHandshake,

    /// Received handshake event, even though it was not expected.
    #[error("Unexpected handshake event")]
    UnexpectedHandshake,

    /// Unable to parse SSE data.
    #[error("Deserialization error: {0}")]
    DeserializizationError(#[from] serde_json::Error),

    /// Connection must be stopped, as received shutdown event.
    #[error("Node shutdown")]
    NodeShutdown,
}
