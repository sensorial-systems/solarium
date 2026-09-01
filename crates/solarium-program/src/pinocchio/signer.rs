use crate::{prelude::*, Account, AccountInfo, Check};
use pinocchio_system::instructions::Transfer;

#[derive(Clone, Copy)]
pub struct Signer<'a> {
    pub info: AccountInfo<'a>,
}

impl<'a> Signer<'a> {
    pub fn address(&self) -> Pubkey {
        self.info.key()
    }

    pub fn transfer<T>(&self, amount: u64, to: &Account<'a, T>) -> Result<()> {
        Transfer {
            from: self.info.view(),
            to: to.info.view(),
            lamports: amount,
        }
        .invoke()?;
        Ok(())
    }
}

impl<'a> TryFrom<&'a mut pinocchio::AccountView> for Signer<'a> {
    type Error = Error;

    fn try_from(view: &'a mut pinocchio::AccountView) -> Result<Self> {
        let info = AccountInfo::new(view);
        if !info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature.into());
        }
        Ok(Self { info })
    }
}

impl Check for Signer<'_> {
    fn check(&self) -> Result<()> {
        if self.info.is_signer() {
            Ok(())
        } else {
            Err(ProgramError::MissingRequiredSignature.into())
        }
    }
}
