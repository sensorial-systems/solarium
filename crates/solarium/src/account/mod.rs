use crate::{prelude::*, Guard};
use solana_program::account_info::AccountInfo;

#[derive(Clone)]
pub struct Account<'a, T = ()> {
    /// The account info.
    pub info: &'a AccountInfo<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: borsh::BorshSerialize + borsh::BorshDeserialize> DataAccess<'a, T> for &mut Account<'a, T> {
    fn data(self) -> Result<Guard<'a, T>> {
        let data: T = {
            let data = self.info.try_borrow_mut_data()?;
            let data = &mut &data[..];
            borsh::BorshDeserialize::deserialize(data)?
        };
        
        Ok(Guard { account: self.info, data })
    }
}

impl<'a, T> TryFrom<&'a AccountInfo<'a>> for Account<'a, T> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        let phantom = Default::default();
        Ok(Self { info, phantom })
    }
}
