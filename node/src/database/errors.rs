use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum DatabaseError {
//     #[error("Failed to find object in database")]
//     NotFound(),
//     #[error(transparent)]
//     InternalError(#[from] InternalServerError),
// }

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to connect using pool.")]
    PoolConnectionError(#[from] deadpool::managed::PoolError<deadpool_diesel::Error>),
    #[error("Failed to insert")]
    InsertError(#[from] deadpool_diesel::InteractError),
    #[error("Error in diesel library")]
    ResultError(#[from] diesel::result::Error)
}