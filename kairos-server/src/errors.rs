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
