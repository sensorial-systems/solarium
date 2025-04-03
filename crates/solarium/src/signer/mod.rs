use crate::prelude::*;
use crate::Check;
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

#[derive(Clone)]
pub struct Signer<'a, T = ()> {
    /// The account data.
    pub data: T,
    /// The account info.
    pub info: AccountInfo<'a>,
}

impl<'a> TryFrom<AccountInfo<'a>> for Signer<'a> {
    type Error = Error;

    fn try_from(value: AccountInfo<'a>) -> Result<Self> {
        if !value.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(Self { data: (), info: value })
    }
}

impl<'a, T> Check for Signer<'a, T> {
    fn check(&self) -> Result<()> {
        if self.info.is_signer {
            Ok(())
        } else {
            Err(ProgramError::MissingRequiredSignature)
        }
    }
}
