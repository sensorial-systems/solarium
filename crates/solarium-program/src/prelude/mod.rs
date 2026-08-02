pub use solarium::prelude::*;
pub use crate::result::*;
pub use crate::PubkeyExt;

#[cfg(not(target_arch = "wasm32"))]
pub use solana_program;
#[cfg(not(target_arch = "wasm32"))]
pub use solana_program::msg;
#[cfg(not(target_arch = "wasm32"))]
pub use solana_program::program_error::ProgramError;
#[cfg(not(target_arch = "wasm32"))]
pub use solana_program::pubkey::Pubkey;

#[cfg(target_arch = "wasm32")]
pub use crate::{msg, solana_program, Account, Program, ProgramError, Pubkey, Signer};

#[cfg(not(target_arch = "wasm32"))]
pub use crate::{AccountInitialization, Check, DataAccess, ResizableDataAccess};