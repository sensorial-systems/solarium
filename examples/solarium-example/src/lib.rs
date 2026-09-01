#![allow(unexpected_cfgs)]

use solana_program::msg;
use solarium::prelude::*;
use solarium_program::prelude::*;
use solarium_program::{Account, Program, Signer};

/// Example PDA account storing a simple message
#[account(pda)]
#[derive(Debug, Default)]
pub struct ExampleData {
    /// Arbitrary message set by the program
    pub message: String,
}

#[derive(Default)]
pub struct ExampleSeeds {}

impl Seeds for ExampleSeeds {
    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![b"example".to_vec()]
    }
}

#[program]
impl Example {
    pub fn initialize<'a>(
        &self,
        payer: &Signer<'a>,
        data: &mut Account<'a, ExampleData>,
        system_program: &Program<'a>,
    ) -> Result<()> {
        msg!("Initializing Example PDA");
        data.allocate(payer, ExampleSeeds::default(), system_program, 256)?;
        Ok(())
    }

    pub fn set_message(
        &self,
        _payer: &Signer,
        data: &mut Account<ExampleData>,
        message: String,
    ) -> Result<()> {
        msg!("Setting message: {}", message);
        data.data()?.message = message;
        Ok(())
    }
}
