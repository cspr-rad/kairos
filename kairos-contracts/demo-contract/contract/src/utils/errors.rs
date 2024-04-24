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
    InvalidTransactionData = 11,
    InvalidTransactionType = 12,
    FailedToParsePublicKey = 13,
    InvalidTransactionSigner = 14,
    FailedToParseTransactionAmount = 15,
    OverflowTransactionAmount = 16,
    InvalidTransactionAmount = 17,
    FailedToCreateSigner = 18,
    InvalidTransactionSignature = 19,
}

impl From<DepositError> for ApiError {
    fn from(error: DepositError) -> Self {
        ApiError::User(error as u16)
    }
}
