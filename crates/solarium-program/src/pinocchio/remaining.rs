use crate::AccountInfo;

/// Every account after the ones a method names — for an instruction whose account list runs on,
/// as one that takes a pair of accounts per token it touches.
///
/// Only the last parameter can be `Remaining`, since it takes all that is left.
#[derive(Clone)]
pub struct Remaining<'a> {
    pub accounts: Vec<AccountInfo<'a>>,
}

impl<'a> Remaining<'a> {
    /// Takes the rest of `accounts`, leaving it empty.
    pub fn rest(accounts: &mut core::slice::IterMut<'a, pinocchio::AccountView>) -> Self {
        Self {
            accounts: accounts.map(AccountInfo::new).collect(),
        }
    }
}

impl<'a> core::ops::Deref for Remaining<'a> {
    type Target = [AccountInfo<'a>];

    fn deref(&self) -> &Self::Target {
        &self.accounts
    }
}
