use crate::prelude::*;

use solana_program::account_info::AccountInfo;

pub struct GuardMut<'a, T: BorshSerialize> {
    pub account: &'a AccountInfo<'a>,
    pub data: T,
}

impl<'a, T: BorshSerialize> Drop for GuardMut<'a, T> {
    fn drop(&mut self) {
        if let Ok(mut data) = self.account.try_borrow_mut_data() {
            if let Ok(serialize_data) = crate::prelude::borsh::to_vec(&self.data) {
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
