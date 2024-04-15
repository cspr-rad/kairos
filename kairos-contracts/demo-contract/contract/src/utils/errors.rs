//! Error handling on the Casper platform.
use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum DepositError {
    InvalidContext = 0,
    MissingKey = 1,
    FailedToGetArgBytes = 2,
    MissingOptionalArgument = 3,
    AlreadyInitialized = 4,
    MissingKeyDepositPurse = 5,
    MissingKeyMostRecentDepositCounter = 6,
    MissingKeyLastProcessedDepositCounter = 7,
    MissingKeyDepositEventDict = 8,
    FailedToCreateDepositDict = 9,
    FailedToReturnContractPurseAsReference = 10,
}

impl From<DepositError> for ApiError {
    fn from(error: DepositError) -> Self {
        ApiError::User(error as u16)
    }
}
