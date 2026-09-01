use crate::prelude::*;
use crate::{pinocchio_backend::address, Account, Program, Signer};
use pinocchio::cpi::{Seed, Signer as CpiSigner};
use pinocchio::sysvars::{rent::Rent, Sysvar};
use pinocchio_system::instructions::CreateAccount;

pub trait AccountInitialization<'a, T> {
    fn initialize<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
    ) -> Result<()>
    where
        T: Default;

    fn initialize_with_data<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
        initial: T,
    ) -> Result<()>
    where
        T: Discriminator;

    fn allocate<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
        space: usize,
    ) -> Result<()>;
}

impl<'a, T: Initialization> AccountInitialization<'a, T> for &mut Account<'a, T> {
    fn initialize<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
    ) -> Result<()>
    where
        T: Default,
    {
        self.initialize_with_data(payer, seeds, system_program, T::default())
    }

    fn initialize_with_data<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
        initial: T,
    ) -> Result<()>
    where
        T: Discriminator,
    {
        if self.info.lamports() > 0 {
            return Ok(());
        }
        let initial_data = initial.to_account_bytes()?;
        create(self, payer, seeds, system_program, initial_data.len())?;
        self.info.try_borrow_mut_data()?[..initial_data.len()].copy_from_slice(&initial_data);
        Ok(())
    }

    fn allocate<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
        space: usize,
    ) -> Result<()> {
        if self.info.lamports() > 0 {
            return Ok(());
        }
        let space = space + crate::DISCRIMINATOR_LEN;
        create(self, payer, seeds, system_program, space)?;
        self.info.try_borrow_mut_data()?[..crate::DISCRIMINATOR_LEN]
            .copy_from_slice(&T::DISCRIMINATOR);
        Ok(())
    }
}

fn create<'a, T: Initialization, S: Seeds>(
    account: &mut Account<'a, T>,
    payer: &Signer<'a>,
    seeds: S,
    _system_program: &Program<'a>,
    space: usize,
) -> Result<()> {
    let seeds = seeds.seeds();
    let seed_bytes: Vec<&[u8]> = seeds.iter().map(Vec::as_slice).collect();
    let (expected, bump) = crate::find_program_address(&seed_bytes, T::owner());
    if account.info.key() != expected {
        return Err(ProgramError::InvalidSeeds.into());
    }
    let bump_seed = [bump];
    let mut signer_seeds: Vec<Seed<'_>> = seed_bytes.iter().map(|seed| Seed::from(*seed)).collect();
    signer_seeds.push(Seed::from(&bump_seed));
    let signer = CpiSigner::from(signer_seeds.as_slice());

    let rent = Rent::get()?;
    CreateAccount {
        from: payer.info.view(),
        to: account.info.view(),
        lamports: rent.try_minimum_balance(space)?,
        space: space as u64,
        owner: &address(T::owner()),
    }
    .invoke_signed(&[signer])?;
    Ok(())
}
