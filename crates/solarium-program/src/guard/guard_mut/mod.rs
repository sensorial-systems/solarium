use crate::{Account, Program, Signer, prelude::*};

use solana_program::account_info::AccountInfo;
use solana_program::msg;

pub struct GuardMut<'a, T: BorshSerialize> {
    pub account: &'a AccountInfo<'a>,
    pub resize: Option<(Signer<'a>, Program<'a>)>,
    pub data: T,
}

impl<'a, T: BorshSerialize> Drop for GuardMut<'a, T> {
    fn drop(&mut self) {
        if let Ok(serialize_data) = crate::prelude::borsh::to_vec(&self.data) {
            if let Some((signer, program)) = self.resize {
                if let Err(e) = Account::<'a, T>::account_realloc_to(self.account, &signer, &program, serialize_data.len(), false) {
                    msg!("Error reallocating account: {}", e);
                }
            }
            if let Ok(mut data) = self.account.try_borrow_mut_data() {
                let len = core::cmp::min(serialize_data.len(), data.len());
                data[..len].copy_from_slice(&serialize_data[..len]);    
            }
        }
    }
}

impl<'a, T: BorshSerialize> std::ops::Deref for GuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: BorshSerialize> std::ops::DerefMut for GuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
