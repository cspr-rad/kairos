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
    InvalidTransactionData = 9,
    InvalidTransactionType = 10,
    FailedToParsePublicKey = 11,
    InvalidTransactionSigner = 12,
    FailedToParseTransactionAmount = 13,
    OverflowTransactionAmount = 14,
    InvalidTransactionAmount = 15,
    FailedToCreateSigner = 16,
    InvalidTransactionSignature = 17,
}

impl From<DepositError> for ApiError {
    fn from(error: DepositError) -> Self {
        ApiError::User(error as u16)
    }
}
