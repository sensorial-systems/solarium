use borsh::BorshSerialize;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::{prelude::*, Owner, Pda, Space};
use crate::{Signer, Account, Program};

pub trait AccountInitialization {
    fn initialize<'a>(
        account: &mut Account<'a>,
        payer: &Signer<'a>,
        sysvar_rent: &Account<'a>,
        system_program: &Program<'a>
    ) -> Result<()>;
}

impl<T: Default + Space + Pda + Owner + BorshSerialize> AccountInitialization for T {
    fn initialize<'a>(account: &mut Account<'a>, payer: &Signer<'a>, sysvar_rent: &Account<'a>, system_program: &Program<'a>) -> Result<()> {
        let account_data = T::default();
        let account_data = crate::prelude::borsh::to_vec(&account_data).unwrap();

        let (account_pda, bump_seed) = Pubkey::find_program_address(T::seeds(), T::owner());
        let rent = Rent::from_account_info(sysvar_rent.info)?;
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
            &[payer.info.clone(), account.info.clone(), system_program.info.clone()],
            &[seeds.as_slice()],
        )?;

        let mut data = account.info.try_borrow_mut_data().unwrap();
        let serialize_data = crate::prelude::borsh::to_vec(&account_data).unwrap();
        data[..serialize_data.len()].copy_from_slice(&serialize_data);
        Ok(())
    }
}
