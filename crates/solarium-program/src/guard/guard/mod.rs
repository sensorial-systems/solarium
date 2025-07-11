use crate::prelude::*;
use solana_program::account_info::AccountInfo;

pub struct Guard<'a, T> {
    pub account: &'a AccountInfo,
    pub data: T,
}

impl<'a, T> core::ops::Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
