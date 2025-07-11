use crate::prelude::*;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey, clock, rent};

pub trait Context<'a> {
    fn get_accounts(&'a self) -> &'a [AccountInfo];

    fn get_rent(&'a self) -> &'a AccountInfo {
        self.get_account(&Pubkey(rent::ID))
    }

    fn get_system_program(&'a self) -> &'a AccountInfo {
        self.get_account(&Pubkey(solana_program::system_program::ID))
    }

    fn get_clock(&'a self) -> &'a AccountInfo {
        self.get_account(&Pubkey(clock::ID))
    }

    fn get_account(&'a self, key: &Pubkey) -> &'a AccountInfo {
        self.get_accounts().iter().find(|account| account.key() == &key.0).unwrap()
    }
}