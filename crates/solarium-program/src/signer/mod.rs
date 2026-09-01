use crate::prelude::*;
use crate::Account;
use crate::Check;
use solana_program::program::invoke;
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

#[derive(Clone, Copy)]
pub struct Signer<'a> {
    /// The account info.
    pub info: &'a AccountInfo<'a>,
}

impl<'a> Signer<'a> {
    pub fn address(&self) -> Pubkey {
        *self.info.key
    }

    pub fn transfer<T>(&self, amount: u64, to: &Account<'a, T>) -> Result<()> {
        let instruction = crate::system_instruction::transfer(&self.info.key, &to.info.key, amount);
        invoke(&instruction, &[self.info.clone(), to.info.clone()])?;
        Ok(())
    }
}

impl<'a> TryFrom<&'a AccountInfo<'a>> for Signer<'a> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        if !info.is_signer {
            return Err(ProgramError::MissingRequiredSignature.into());
        }
        Ok(Self { info })
    }
}

impl<'a> Check for Signer<'a> {
    fn check(&self) -> Result<()> {
        if self.info.is_signer {
            Ok(())
        } else {
            Err(ProgramError::MissingRequiredSignature.into())
        }
    }
}
