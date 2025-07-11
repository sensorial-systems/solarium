use solarium_program::prelude::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Program error: {0}")]
    ProgramError(solana_program::program_error::ProgramError),
    #[error("Client error: {0}")]
    ClientError(solana_client::client_error::ClientError),
    #[error("Subscription error: {0}")]
    SubscriptionError(solana_client::pubsub_client::PubsubClientError),
    #[error("IO error: {0}")]
    IoError(std::io::Error),
}

impl From<solana_program::program_error::ProgramError> for Error {
    fn from(error: solana_program::program_error::ProgramError) -> Self {
        Error::ProgramError(error)
    }
}

impl From<solana_client::client_error::ClientError> for Error {
    fn from(error: solana_client::client_error::ClientError) -> Self {
        Error::ClientError(error)
    }
}

impl From<solana_client::pubsub_client::PubsubClientError> for Error {
    fn from(error: solana_client::pubsub_client::PubsubClientError) -> Self {
        Error::SubscriptionError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<Error> for solana_program::program_error::ProgramError {
    fn from(error: Error) -> Self {
        match error {
            Error::ProgramError(error) => error,
            Error::IoError(error) => solana_program::program_error::ProgramError::BorshIoError(error.to_string()),
            Error::ClientError(error) => solana_program::program_error::ProgramError::BorshIoError(error.to_string()),
            Error::SubscriptionError(error) => solana_program::program_error::ProgramError::BorshIoError(error.to_string()),
        }
    }
}