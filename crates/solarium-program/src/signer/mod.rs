use crate::prelude::*;
use crate::Check;
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

#[derive(Clone)]
pub struct Signer<'a> {
    /// The account info.
    pub info: &'a AccountInfo,
}

impl<'a> TryFrom<&'a AccountInfo> for Signer<'a> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo) -> Result<Self> {
        if !info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature.into());
        }
        Ok(Self { info })
    }
}

impl<'a> Check for Signer<'a> {
    fn check(&self) -> Result<()> {
        if self.info.is_signer() {
            Ok(())
        } else {
            Err(ProgramError::MissingRequiredSignature.into())
        }
    }
}
