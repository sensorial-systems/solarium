use crate::{AccountInfo, Pubkey};

pub trait Context<'a> {
    fn get_accounts(&'a self) -> &'a [AccountInfo<'a>];

    fn get_rent(&'a self) -> &'a AccountInfo<'a> {
        self.get_account(&solana_sdk_ids::sysvar::rent::ID)
    }

    fn get_system_program(&'a self) -> &'a AccountInfo<'a> {
        self.get_account(&solana_sdk_ids::system_program::ID)
    }

    fn get_clock(&'a self) -> &'a AccountInfo<'a> {
        self.get_account(&solana_sdk_ids::sysvar::clock::ID)
    }

    fn get_account(&'a self, key: &Pubkey) -> &'a AccountInfo<'a> {
        self.get_accounts()
            .iter()
            .find(|account| account.key() == *key)
            .unwrap()
    }
}
