#![allow(unexpected_cfgs)]

use solana_program::msg;
use solarium::prelude::*;
use solarium_program::prelude::*;
use solarium_program::{Account, Program, Signer};

// PDA storing file metadata and a dynamically-sized byte vector
#[account(pda)]
#[derive(Debug)]
pub struct FileAccount {
    pub expected_crc32: u32,
    pub file_size: u64,
    pub written_crc32: u32,
    pub bytes: Vec<u8>,
}

impl Default for FileAccount {
    fn default() -> Self {
        Self { expected_crc32: 0, file_size: 0, written_crc32: 0, bytes: Vec::new() }
    }
}

#[derive(Default)]
pub struct FileSeeds {}

impl Seeds for FileSeeds {
    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![b"file".to_vec()]
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InitArgs {
    pub expected_crc32: u32,
    pub file_size: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UploadChunkArgs {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AllocateArgs {
    pub space: u64,
}

#[program]
impl FileUploadExample {
    pub fn initialize<'a>(
        &self,
        payer: &Signer<'a>,
        file: &mut Account<'a, FileAccount>,
        system_program: &Program<'a>,
        args: InitArgs,
    ) -> Result<()> {
        msg!("Initialize FileAccount PDA with dynamic size");

        // Prepare initial data with requested capacity
        let initial = FileAccount {
            expected_crc32: args.expected_crc32,
            file_size: args.file_size,
            written_crc32: 0,
            bytes: vec![0u8; args.file_size as usize],
        };
        file.initialize_with_data(payer, FileSeeds::default(), system_program, initial)?;
        Ok(())
    }

    pub fn upload_chunk(&self, _payer: &Signer, file: &mut Account<FileAccount>, args: UploadChunkArgs) -> Result<()> {
        msg!("Upload chunk at offset {} ({} bytes)", args.offset, args.data.len());
        let mut acc = file.data()?;
        let end = args.offset as usize + args.data.len();
        if args.offset as u64 > acc.file_size || (end as u64) > acc.file_size || end > acc.bytes.len() {
            return Err(solana_program::program_error::ProgramError::InvalidInstructionData.into());
        }
        acc.bytes[args.offset as usize..end].copy_from_slice(&args.data);
        Ok(())
    }

    pub fn check_crc(&self, _payer: &Signer, file: &mut Account<FileAccount>) -> Result<()> {
        msg!("Check CRC32");
        let mut acc = file.data()?;
        let mut crc: u32 = 0xFFFF_FFFF;
        let remaining = acc.file_size as usize;
        for &byte in acc.bytes.iter().take(remaining) {
            let mut c = (crc ^ (byte as u32)) & 0xFF;
            for _ in 0..8 {
                c = if (c & 1) != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
            }
            crc = (crc >> 8) ^ c;
        }
        acc.written_crc32 = !crc;
        if acc.written_crc32 != acc.expected_crc32 {
            return Err(solana_program::program_error::ProgramError::InvalidAccountData.into());
        }
        Ok(())
    }

    pub fn allocate<'a>(
        &self,
        payer: &Signer<'a>,
        file: &mut Account<'a, FileAccount>,
        system_program: &Program<'a>,
        args: AllocateArgs,
    ) -> Result<()> {
        msg!("Allocate PDA space only: {} bytes", args.space);
        file.allocate(payer, FileSeeds::default(), system_program, args.space as usize)?;
        Ok(())
    }
}


