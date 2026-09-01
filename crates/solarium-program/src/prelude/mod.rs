pub use crate::result::*;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
pub use crate::AccountInfoExt;
pub use crate::PubkeyExt;
pub use solarium::prelude::*;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
pub use solana_program;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
pub use solana_program::account_info::AccountInfo;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
pub use solana_program::msg;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
pub use solana_program::program_error::ProgramError;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
pub use solana_program::pubkey::Pubkey;

#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
pub use crate::{msg, pinocchio, AccountInfo, ProgramError, Pubkey};

#[cfg(target_arch = "wasm32")]
pub use crate::{msg, solana_program, Account, Program, ProgramError, Pubkey, Signer};

#[cfg(not(target_arch = "wasm32"))]
pub use crate::{AccountInitialization, Check, DataAccess, ResizableDataAccess};
