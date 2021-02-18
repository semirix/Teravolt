use thiserror::Error;

/// An error type used by Teravolt.
#[derive(Error, Debug)]
pub enum TeravoltError {
    #[error("Could not initialise Teravolt: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Something happened")]
    Other,
}
