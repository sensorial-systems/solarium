use pinocchio::{account_info::AccountInfo, instruction::{AccountMeta, Instruction}, pubkey::Pubkey};

pub use pinocchio_system::*;


pub struct CreateAccountInstruction<'pubkey> {
    pub account_metas: [AccountMeta<'pubkey>; 2],
    pub data: [u8; 52],
}

pub fn create_account<'pubkey>(from: &'pubkey AccountInfo, to: &'pubkey AccountInfo, lamports: u64, space: u64, owner: &'pubkey Pubkey) -> CreateAccountInstruction<'pubkey> {
    let account_metas: [AccountMeta; 2] = [
        AccountMeta::writable_signer(from.key()),
        AccountMeta::writable_signer(to.key()),
    ];

    let mut data = [0; 52];
    data[4..12].copy_from_slice(&lamports.to_le_bytes());
    data[12..20].copy_from_slice(&space.to_le_bytes());
    data[20..52].copy_from_slice(owner.as_ref());

    CreateAccountInstruction {
        account_metas,
        data,
    }
}

impl<'pubkey> CreateAccountInstruction<'pubkey> {
    pub fn as_instruction<'accounts, 'program_id, 'data>(&'pubkey self) -> Instruction<'pubkey, 'accounts, 'program_id, 'data>
    where
        'pubkey: 'program_id,
        'pubkey: 'data,
    {
        Instruction {
            program_id: &pinocchio_system::ID,
            accounts: &self.account_metas,
            data: &self.data,
        }
    }
}