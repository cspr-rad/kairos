use deadpool_diesel::Error as DDError;
use deadpool_diesel::InteractError as DDIError;
use diesel::result::Error as DieselError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("There was an issue connecting to the database: {0}")]
    ConnectionError(#[from] DDError),
    #[error("There was an error in sync code executing in a separate thread: {0}")]
    InteractError(#[from] DDIError),
    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool::managed::PoolError<deadpool_diesel::Error>),
    #[error("Diesel error: {0}")]
    DieselError(#[from] DieselError),
}
