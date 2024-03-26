//! Error handling on the Casper platform.
use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum DepositError {
    InvalidContext = 0,
    MissingKey = 1,
    InvalidAdminList = 2,
    InvalidNoneList = 3,
    FailedToGetArgBytes = 4,
    Phantom = 5,
    InsufficientRights = 6,
    AlreadyInitialized = 7,
}

impl From<DepositError> for ApiError {
    fn from(error: DepositError) -> Self {
        ApiError::User(error as u16)
    }
}
