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
    MissingOptionalArgument = 5,
    InsufficientRights = 6,
    AlreadyInitialized = 7,
    MissingKeyDepositPurse = 8,
    MissingKeyMostRecentDepositCounter = 9,
    MissingKeyLastProcessedDepositCounter = 10,
    MissingKeyDepositEventDict = 11,
    FailedToCreateSecurityBadgesDict = 12,
    FailedToCreateDepositDict = 13,
    FailedToReturnContractPurseAsReference = 14,
}

impl From<DepositError> for ApiError {
    fn from(error: DepositError) -> Self {
        ApiError::User(error as u16)
    }
}
