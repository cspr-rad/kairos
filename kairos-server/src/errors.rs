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
    pub fn set_status(err: impl Into<Self>, status: StatusCode) -> Self {
        let mut err = err.into();
        err.status = Some(status);
        err
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

/// Replace with `!` when stabilized
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Never;
impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl From<Never> for AppErr {
    fn from(_: Never) -> Self {
        unreachable!("Never was constructed")
    }
}
