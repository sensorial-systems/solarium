use crate::{prelude::*, AccountUpdate};
use borsh::BorshSerialize;
use solana_program::account_info::AccountInfo;

#[derive(Clone)]
pub struct Account<'a, T = ()> {
    /// The account data.
    pub data: T,
    /// The account info.
    pub info: &'a AccountInfo<'a>,
}

impl<'a, T: borsh::BorshDeserialize> TryFrom<&'a AccountInfo<'a>> for Account<'a, T> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        let data: T = {
            let data = info.try_borrow_mut_data()?;
            let data = &mut &data[..];
            borsh::BorshDeserialize::deserialize(data)?
        };
        Ok(Self { data, info })
    }
}

impl<'a, T: BorshSerialize> AccountUpdate for &mut Account<'a, T> {
    fn update(self) -> Result<()> {
        let mut data = self.info.try_borrow_mut_data().unwrap();
        let serialize_data = crate::prelude::borsh::to_vec(&self.data).unwrap();
        data[..serialize_data.len()].copy_from_slice(&serialize_data);
        Ok(())
    }
}
