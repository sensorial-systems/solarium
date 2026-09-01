use crate::{prelude::*, Guard, GuardMut, Program, Signer};
use core::cell::{Ref, RefMut};
use solana_program::account_info::AccountInfo;

#[derive(Clone)]
pub struct Account<'a, T = ()> {
    /// The account info.
    pub info: &'a AccountInfo<'a>,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T> Account<'a, T> {
    pub fn new(info: &'a AccountInfo<'a>) -> Self {
        Self {
            info,
            phantom: Default::default(),
        }
    }

    /// Reads this account, refusing one of another type.
    ///
    /// The tag is checked rather than assumed: the runtime hands a program whatever accounts the
    /// transaction named, and without the check an account of one of the program's other types
    /// would be read as this one wherever its bytes happened to fit.
    pub fn deserialize(&self) -> Result<T>
    where
        T: Discriminator,
    {
        let data = self.info.try_borrow_data()?;
        Ok(T::from_account_bytes(&data)?)
    }

    pub fn bytes(&self) -> Result<Ref<'a, [u8]>> {
        let data = self.info.try_borrow_data()?;
        Ok(Ref::map(data, |d| &d[..]))
    }

    pub fn bytes_mut(&self) -> Result<RefMut<'a, [u8]>> {
        let data = self.info.try_borrow_mut_data()?;
        Ok(RefMut::map(data, |d| &mut d[..]))
    }

    pub fn account_realloc_to(
        account: &'a AccountInfo<'a>,
        payer: &Signer<'a>,
        system_program: &Program<'a>,
        new_len: usize,
        zero_init: bool,
    ) -> Result<()> {
        use solana_program::{program::invoke, rent::Rent, sysvar::Sysvar};
        // Top up lamports if needed to maintain rent exemption at new size
        let rent = Rent::get()?;
        let required = rent.minimum_balance(new_len);
        let current = account.lamports();
        if required > current {
            let ix = solana_program::system_instruction::transfer(
                &payer.info.signer_key().unwrap(),
                account.key,
                required - current,
            );
            invoke(
                &ix,
                &[
                    payer.info.clone(),
                    account.clone(),
                    system_program.info.clone(),
                ],
            )?;
        }
        let _ = zero_init;
        account.resize(new_len)?;
        Ok(())
    }

    pub fn realloc_to(
        &self,
        payer: &Signer<'a>,
        system_program: &Program<'a>,
        new_len: usize,
        zero_init: bool,
    ) -> Result<()> {
        Self::account_realloc_to(self.info, payer, system_program, new_len, zero_init)
    }
}

impl<'a, T> TryFrom<&'a AccountInfo<'a>> for Account<'a, T> {
    type Error = Error;

    fn try_from(info: &'a AccountInfo<'a>) -> Result<Self> {
        let phantom = Default::default();
        Ok(Self { info, phantom })
    }
}

impl<'a, T: Discriminator> DataAccess<'a, T> for &mut Account<'a, T> {
    type Output = Result<GuardMut<'a, T>>;
    fn data(self) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;
        let resize = Default::default();

        Ok(GuardMut {
            account,
            data,
            resize,
        })
    }
}

impl<'a, T: Discriminator> ResizableDataAccess<'a, T> for &mut Account<'a, T> {
    type Output = Result<GuardMut<'a, T>>;
    fn resizeable_data(self, payer: &Signer<'a>, program: &Program<'a>) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;
        let resize = Some((*payer, *program));
        Ok(GuardMut {
            account,
            data,
            resize,
        })
    }
}

impl<'a, T: Discriminator> DataAccess<'a, T> for &Account<'a, T> {
    type Output = Result<Guard<'a, T>>;
    fn data(self) -> Self::Output {
        let account = self.info;
        let data = self.deserialize()?;

        Ok(Guard { account, data })
    }
}
