use thiserror::Error;

/// An error type used by Teravolt.
#[derive(Error, Debug)]
pub enum TeravoltError {
    #[error("Config Error: {0:?}")]
    ConfigError(#[from] ErrorMessage),
    #[error("Could not initialise Teravolt: {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Something happened")]
    Unknown,
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct ErrorMessage(pub String);
