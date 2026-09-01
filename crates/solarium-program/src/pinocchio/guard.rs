use crate::{prelude::*, Account, AccountInfo, Program, Signer};
use solana_program_log::log as msg;

pub struct Guard<'a, T> {
    pub account: AccountInfo<'a>,
    pub data: T,
}

impl<T> core::ops::Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct GuardMut<'a, T: Discriminator> {
    pub account: AccountInfo<'a>,
    pub resize: Option<(Signer<'a>, Program<'a>)>,
    pub data: T,
}

impl<'a, T: Discriminator> Drop for GuardMut<'a, T> {
    fn drop(&mut self) {
        if let Ok(serialized) = self.data.to_account_bytes() {
            if let Some((signer, program)) = self.resize {
                if let Err(error) = Account::<'a, T>::account_realloc_to(
                    &mut self.account,
                    &signer,
                    &program,
                    serialized.len(),
                    false,
                ) {
                    let _ = error;
                    msg!("Error reallocating account");
                }
            }
            if let Ok(mut data) = self.account.try_borrow_mut_data() {
                let len = core::cmp::min(serialized.len(), data.len());
                data[..len].copy_from_slice(&serialized[..len]);
            }
        }
    }
}

impl<T: Discriminator> core::ops::Deref for GuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Discriminator> core::ops::DerefMut for GuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
