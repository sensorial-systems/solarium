use crate::prelude::*;
use crate::{Signer, Account, Program};

use solarium_pinocchio::instruction::{Seed};
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use alloc::vec::Vec;

pub trait AccountInitialization<'a, T> {
    fn initialize<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>
    ) -> Result<()>;
}

impl<'a, T: Initialization> AccountInitialization<'a, T> for &mut Account<'a, T> {
    fn initialize<S: Seeds>(
        self,
        payer: &Signer<'a>,
        seeds: S,
        system_program: &Program<'a>
    ) -> Result<()> {
        if self.info.lamports() > 0 {
            return Ok(());
        }

        let seeds = seeds.seeds();
        let seeds = seeds.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
        let seeds = seeds.as_slice();

        let account_data = T::default();
        let account_data = crate::prelude::borsh::to_vec(&account_data).unwrap();

        let (_account_pda, bump_seed) = Pubkey::find_program_address(seeds, T::owner());
        // TODO: Assert account_pda is equal to self.info.key()
        let rent = Rent::get()?;
        let data_size = T::space();
        let rent_amount = rent.minimum_balance(data_size);
        let instruction = solana_program::system_program::create_account(
            &payer.info,
            self.info,
            rent_amount,
            data_size as u64,
            T::owner(),
        );

        let bump_seed = [bump_seed];
        let mut seeds_data = Vec::with_capacity(seeds.len() + 1);
        seeds_data.extend_from_slice(seeds);
        seeds_data.push(&bump_seed);
    
        invoke_signed(
            &instruction.as_instruction(),
            &[payer.info, self.info, system_program.info],
            &[solana_program::instruction::Signer::from(&[Seed::from(&bump_seed)])],
        )?;

        let mut data = self.info.try_borrow_mut_data().unwrap();
        let serialize_data = crate::prelude::borsh::to_vec(&account_data).unwrap();
        data[..serialize_data.len()].copy_from_slice(&serialize_data);
        Ok(())
    }
}