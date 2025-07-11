use crate::prelude::*;

use solana_program::account_info::AccountInfo;

pub struct GuardMut<'a, T: BorshSerialize> {
    pub account: &'a AccountInfo,
    pub data: T,
}

impl<'a, T: BorshSerialize> Drop for GuardMut<'a, T> {
    fn drop(&mut self) {
        let mut data = self.account.try_borrow_mut_data().unwrap();
        let serialize_data = crate::prelude::borsh::to_vec(&self.data).unwrap();
        data[..serialize_data.len()].copy_from_slice(&serialize_data);
    }
}

impl<'a, T: BorshSerialize> core::ops::Deref for GuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: BorshSerialize> core::ops::DerefMut for GuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
