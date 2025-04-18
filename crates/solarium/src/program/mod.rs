use crate::prelude::*;
use solana_program::{account_info::AccountInfo, program_error::ProgramError};


pub struct Program<'a> {
    pub info: &'a AccountInfo<'a>,
}

impl<'a> TryFrom<&'a AccountInfo<'a>> for Program<'a> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        if info.executable {
            Ok(Self { info })
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
}