use solana_program::account_info::AccountInfo;

pub struct Guard<'a, T> {
    pub account: &'a AccountInfo<'a>,
    pub data: T,
}

impl<'a, T> std::ops::Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
