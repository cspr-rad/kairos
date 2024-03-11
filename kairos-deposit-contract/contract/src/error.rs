//! Error handling on the casper platform.
use casper_types::ApiError;
pub enum Error {
    InvalidContext,
    User(u16),
}
const INVALIDCONTEXT: u16 = 0u16;
impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        let user_error = match error {
            Error::InvalidContext => INVALIDCONTEXT,
            Error::User(user_error) => user_error,
        };
        ApiError::User(user_error)
    }
}
