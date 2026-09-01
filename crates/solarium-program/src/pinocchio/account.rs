use crate::{prelude::*, AccountInfo, Guard, GuardMut, Program, Signer};
use core::marker::PhantomData;
use pinocchio::{account::Ref, account::RefMut, sysvars::rent::Rent, sysvars::Sysvar, Resize};
use pinocchio_system::instructions::Transfer;

#[derive(Clone)]
pub struct Account<'a, T = ()> {
    pub info: AccountInfo<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Account<'a, T> {
    pub fn new(info: AccountInfo<'a>) -> Self {
        Self {
            info,
            phantom: PhantomData,
        }
    }

    pub fn deserialize(&self) -> Result<T>
    where
        T: Discriminator,
    {
        let data = self.info.try_borrow_data()?;
        Ok(T::from_account_bytes(&data)?)
    }

    pub fn bytes(&self) -> Result<Ref<'_, [u8]>> {
        Ok(self.info.try_borrow_data()?)
    }

    pub fn bytes_mut(&mut self) -> Result<RefMut<'_, [u8]>> {
        Ok(self.info.try_borrow_mut_data()?)
    }

    pub fn account_realloc_to(
        account: &mut AccountInfo<'a>,
        payer: &Signer<'a>,
        _system_program: &Program<'a>,
        new_len: usize,
        _zero_init: bool,
    ) -> Result<()> {
        let rent = Rent::get()?;
        let required = rent.try_minimum_balance(new_len)?;
        let current = account.lamports();
        if required > current {
            Transfer {
                from: payer.info.view(),
                to: account.view(),
                lamports: required - current,
            }
            .invoke()?;
        }
        account.view_mut().resize(new_len)?;
        Ok(())
    }

    pub fn realloc_to(
        &mut self,
        payer: &Signer<'a>,
        system_program: &Program<'a>,
        new_len: usize,
        zero_init: bool,
    ) -> Result<()> {
        Self::account_realloc_to(&mut self.info, payer, system_program, new_len, zero_init)
    }
}

impl<'a, T> TryFrom<&'a mut pinocchio::AccountView> for Account<'a, T> {
    type Error = Error;

    fn try_from(view: &'a mut pinocchio::AccountView) -> Result<Self> {
        Ok(Self::new(AccountInfo::new(view)))
    }
}

impl<'a, T: Discriminator> DataAccess<'a, T> for &mut Account<'a, T> {
    type Output = Result<GuardMut<'a, T>>;

    fn data(self) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;
        Ok(GuardMut {
            account,
            data,
            resize: None,
        })
    }
}

impl<'a, T: Discriminator> ResizableDataAccess<'a, T> for &mut Account<'a, T> {
    type Output = Result<GuardMut<'a, T>>;

    fn resizeable_data(self, payer: &Signer<'a>, program: &Program<'a>) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;
        Ok(GuardMut {
            account,
            data,
            resize: Some((*payer, *program)),
        })
    }
}

impl<'a, T: Discriminator> DataAccess<'a, T> for &Account<'a, T> {
    type Output = Result<Guard<'a, T>>;

    fn data(self) -> Self::Output {
        Ok(Guard {
            account: self.info,
            data: self.deserialize()?,
        })
    }
}
