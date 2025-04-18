use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::prelude::*;
use crate::{Signer, Account, Program};

pub trait AccountInitialization<'a, T> {
    fn initialize(
        self,
        payer: &Signer<'a>,
        system_program: &Program<'a>
    ) -> Result<()>;
}

impl<'a, T: Initialization> AccountInitialization<'a, T> for &mut Account<'a, T> {
    fn initialize(
        self,
        payer: &Signer<'a>,
        system_program: &Program<'a>
    ) -> Result<()> {
        if self.info.lamports() > 0 {
            return Ok(());
        }

        let account_data = T::default();
        let account_data = crate::prelude::borsh::to_vec(&account_data).unwrap();

        let (account_pda, bump_seed) = Pubkey::find_program_address(T::seeds(), T::owner());
        let rent = Rent::get()?;
        let data_size = T::space();
        let rent_amount = rent.minimum_balance(data_size);
        let instruction = solana_program::system_instruction::create_account(
            &payer.info.signer_key().unwrap(),
            &account_pda,
            rent_amount,
            data_size as u64,
            T::owner(),
        );

        let bump_seed = [bump_seed];
        let mut seeds = Vec::with_capacity(T::seeds().len() + 1);
        seeds.extend_from_slice(T::seeds());
        seeds.push(&bump_seed);
    
        invoke_signed(
            &instruction,
            &[payer.info.clone(), self.info.clone(), system_program.info.clone()],
            &[seeds.as_slice()],
        )?;

        let mut data = self.info.try_borrow_mut_data().unwrap();
        let serialize_data = crate::prelude::borsh::to_vec(&account_data).unwrap();
        data[..serialize_data.len()].copy_from_slice(&serialize_data);
        Ok(())
    }
}