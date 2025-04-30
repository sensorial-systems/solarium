use crate::prelude::solana_program::pubkey::Pubkey;

pub trait Seeds {
    fn program() -> &'static Pubkey;
    fn seeds(&self) -> Vec<Vec<u8>>;
}