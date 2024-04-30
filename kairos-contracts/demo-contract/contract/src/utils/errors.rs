//! Error handling on the Casper platform.
use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum DepositError {
    InvalidContext = 0,
    MissingKey = 1,
    FailedToGetArgBytes = 2,
    AlreadyInitialized = 3,
    MissingKeyDepositPurse = 4,
    MissingKeyLastProcessedDepositCounter = 5,
    MissingKeyDepositEventDict = 6,
    FailedToCreateDepositDict = 7,
    FailedToReturnContractPurseAsReference = 8,
}

impl From<DepositError> for ApiError {
    fn from(error: DepositError) -> Self {
        ApiError::User(error as u16)
    }
}
