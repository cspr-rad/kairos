use thiserror::Error;

#[derive(Error, Debug)]
pub enum L1SyncError {
    /// Casper Event Toolkit error.
    #[error("toolkit error: {error}")]
    ToolkitError {
        #[from]
        error: casper_event_toolkit::error::ToolkitError,
    },

    /// Communication error.
    #[error("channel error: {0}")]
    BrokenChannel(String),

    /// Initialization error.
    #[error("Initialization error: {0}")]
    InitializationError(String),

    /// Error that we cannot recover from.
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
