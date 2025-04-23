use crate::{prelude::*, GuardMut, Guard};
use solana_program::account_info::AccountInfo;

#[derive(Clone)]
pub struct Account<'a, T = ()> {
    /// The account info.
    pub info: &'a AccountInfo<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Account<'a, T> {
    pub fn new(info: &'a AccountInfo<'a>) -> Self {
        Self { info, phantom: Default::default() }
    }

    pub fn deserialize(&self) -> Result<T>
    where T: borsh::BorshDeserialize
    {
        let data = self.info.try_borrow_data()?;
        let data = &mut &data[..];
        Ok(borsh::BorshDeserialize::deserialize(data)?)
    }
}

impl<'a, T> TryFrom<&'a AccountInfo<'a>> for Account<'a, T> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        let phantom = Default::default();
        Ok(Self { info, phantom })
    }
}


impl<'a, T: borsh::BorshSerialize + borsh::BorshDeserialize> DataAccess<'a, T> for &mut Account<'a, T> {
    type Output = Result<GuardMut<'a, T>>;
    fn data(self) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;
        
        Ok(GuardMut { account, data })
    }
}

impl<'a, T: borsh::BorshSerialize + borsh::BorshDeserialize> DataAccess<'a, T> for &Account<'a, T> {
    type Output = Result<Guard<'a, T>>;
    fn data(self) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;
        
        Ok(Guard { account, data })
    }
}
