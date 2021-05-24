use thiserror::Error;

/// An error type used by Teravolt.
#[derive(Error, Debug)]
pub enum TeravoltError {
    #[error("Error: {0:?}")]
    GenericError(#[from] ErrorMessage),
    #[error("Could not initialise Teravolt: {0:?}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct ErrorMessage(pub String);
