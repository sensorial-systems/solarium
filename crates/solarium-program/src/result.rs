use crate::prelude::*;

pub type Result<T> = core::result::Result<T, Error>;

pub type Error = solana_program::program_error::ProgramError;
