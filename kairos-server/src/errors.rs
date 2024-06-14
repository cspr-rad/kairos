use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub struct AppErr {
    error: anyhow::Error,
    status: Option<StatusCode>,
}

impl AppErr {
    pub fn new(error: impl Into<anyhow::Error>) -> Self {
        Self {
            error: error.into(),
            status: None,
        }
    }

    pub fn set_status(mut self, status: StatusCode) -> Self {
        self.status = Some(status);
        self
    }
}

impl IntoResponse for AppErr {
    fn into_response(self) -> Response {
        (
            self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            format!("{}", self.error),
        )
            .into_response()
    }
}

impl Deref for AppErr {
    type Target = anyhow::Error;
    fn deref(&self) -> &Self::Target {
        &self.error
    }
}

impl DerefMut for AppErr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.error
    }
}

impl fmt::Display for AppErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            self.error
        )
    }
}

impl std::error::Error for AppErr {}

impl From<anyhow::Error> for AppErr {
    fn from(error: anyhow::Error) -> Self {
        Self {
            error,
            status: None,
        }
    }
}

impl From<kairos_trie::TrieError> for AppErr {
    fn from(error: kairos_trie::TrieError) -> Self {
        Self {
            error: anyhow::Error::msg(error.to_string()),
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl From<casper_client::Error> for AppErr {
    fn from(error: casper_client::Error) -> Self {
        Self {
            error: anyhow::Error::msg(error.to_string()),
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[cfg(feature="database")]
impl From<kairos_data::errors::DBError> for AppErr {
    fn from(error: kairos_data::errors::DBError) -> Self {
        Self {
            error: anyhow::Error::msg(error.to_string()),
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
