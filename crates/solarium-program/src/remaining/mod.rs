use crate::AccountInfo;

/// Every account after the ones a method names — for an instruction whose account list runs on,
/// as one that takes a pair of accounts per token it touches.
///
/// Only the last parameter can be `Remaining`, since it takes all that is left.
#[derive(Clone, Copy)]
pub struct Remaining<'a> {
    pub accounts: &'a [AccountInfo<'a>],
}

impl<'a> Remaining<'a> {
    pub fn new(accounts: &'a [AccountInfo<'a>]) -> Self {
        Self { accounts }
    }
}

impl<'a> core::ops::Deref for Remaining<'a> {
    type Target = [AccountInfo<'a>];

    fn deref(&self) -> &Self::Target {
        self.accounts
    }
}
