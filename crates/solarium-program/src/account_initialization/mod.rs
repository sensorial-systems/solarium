use crate::prelude::*;
use crate::{Signer, Account, Program};

use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

pub trait AccountInitialization<'a, T> {
    fn initialize<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>
    ) -> Result<()>;

    fn initialize_with_data<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
        initial: T,
    ) -> Result<()>
    where T: BorshSerialize + Default;

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
        system_program: &Program<'a>
    ) -> Result<()> {
        self.initialize_with_data(payer, seeds, system_program, T::default())
    }

    fn initialize_with_data<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>,
        initial: T,
    ) -> Result<()>
    where T: BorshSerialize + Default
    {
        if self.info.lamports() > 0 {
            return Ok(());
        }

        let seeds = seeds.seeds();
        let seeds = seeds.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
        let seeds = seeds.as_slice();

        let initial_data = crate::prelude::borsh::to_vec(&initial).unwrap();

        let (account_pda, bump_seed) = Pubkey::find_program_address(seeds, T::owner());
        let rent = Rent::get()?;
        let data_size = initial_data.len();
        let rent_amount = rent.minimum_balance(data_size);
        let instruction = solana_program::system_instruction::create_account(
            &payer.info.signer_key().unwrap(),
            &account_pda,
            rent_amount,
            data_size as u64,
            T::owner(),
        );

        let bump_seed = [bump_seed];
        let mut seeds_data = Vec::with_capacity(seeds.len() + 1);
        seeds_data.extend_from_slice(seeds);
        seeds_data.push(&bump_seed);

        invoke_signed(
            &instruction,
            &[payer.info.clone(), self.info.clone(), system_program.info.clone()],
            &[seeds_data.as_slice()],
        )?;

        let mut data = self.info.try_borrow_mut_data().unwrap();
        data[..initial_data.len()].copy_from_slice(&initial_data);
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

        let seeds = seeds.seeds();
        let seeds = seeds.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
        let seeds = seeds.as_slice();

        let (account_pda, bump_seed) = Pubkey::find_program_address(seeds, T::owner());
        let rent = Rent::get()?;
        let rent_amount = rent.minimum_balance(space);
        let instruction = solana_program::system_instruction::create_account(
            &payer.info.signer_key().unwrap(),
            &account_pda,
            rent_amount,
            space as u64,
            T::owner(),
        );

        let bump_seed = [bump_seed];
        let mut seeds_data = Vec::with_capacity(seeds.len() + 1);
        seeds_data.extend_from_slice(seeds);
        seeds_data.push(&bump_seed);

        invoke_signed(
            &instruction,
            &[payer.info.clone(), self.info.clone(), system_program.info.clone()],
            &[seeds_data.as_slice()],
        )?;

        Ok(())
    }
}