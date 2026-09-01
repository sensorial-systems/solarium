pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Program error: {0:?}")]
    ProgramError(crate::ProgramError),
    #[error("IO error: {0}")]
    IoError(std::io::Error),
}

impl From<crate::ProgramError> for Error {
    fn from(error: crate::ProgramError) -> Self {
        Error::ProgramError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<Error> for crate::ProgramError {
    fn from(error: Error) -> Self {
        match error {
            Error::ProgramError(error) => error,
            #[cfg(not(target_arch = "wasm32"))]
            Error::IoError(error) => crate::ProgramError::BorshIoError(error.to_string()),
            #[cfg(target_arch = "wasm32")]
            Error::IoError(_error) => crate::ProgramError::BorshIoError,
        }
    }
}
