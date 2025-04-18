use solana_program::account_info::AccountInfo;
use borsh::BorshSerialize;

pub struct Guard<'a, T: BorshSerialize> {
    pub account: &'a AccountInfo<'a>,
    pub data: T,
}

impl<'a, T: BorshSerialize> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        let mut data = self.account.try_borrow_mut_data().unwrap();
        let serialize_data = crate::prelude::borsh::to_vec(&self.data).unwrap();
        data[..serialize_data.len()].copy_from_slice(&serialize_data);
    }
}

impl<'a, T: BorshSerialize> std::ops::Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: BorshSerialize> std::ops::DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

