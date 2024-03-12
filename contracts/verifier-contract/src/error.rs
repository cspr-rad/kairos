use casper_types::ApiError;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum RiscZeroError{
    InvalidProof = 0
}
impl From<RiscZeroError> for ApiError{
    fn from(e: RiscZeroError) -> Self{
        ApiError::User(e as u16)
    }
}