pub mod prelude;

pub mod result;

// Features only add: with both backends on, Pinocchio is the one used, as the program macro has it.
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "pinocchio"),
    not(feature = "solana-program-backend")
))]
compile_error!(r#"select either the "solana-program-backend" or "pinocchio" feature"#);

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod account;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/account.rs"]
mod account;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod account_info;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod account_initialization;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/account_initialization.rs"]
mod account_initialization;

#[cfg(not(target_arch = "wasm32"))]
mod check;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod context;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/context.rs"]
mod context;

#[cfg(not(target_arch = "wasm32"))]
mod data_access;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod guard;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/guard.rs"]
mod guard;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod program;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/program.rs"]
mod program;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod remaining;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/remaining.rs"]
mod remaining;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod signer;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/signer.rs"]
mod signer;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
mod system_instruction;

#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/mod.rs"]
mod pinocchio_backend;

#[cfg(not(target_arch = "wasm32"))]
pub use account::*;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
pub use account_info::*;
#[cfg(not(target_arch = "wasm32"))]
pub use account_initialization::*;
#[cfg(not(target_arch = "wasm32"))]
pub use check::*;
#[cfg(not(target_arch = "wasm32"))]
pub use context::*;
#[cfg(not(target_arch = "wasm32"))]
pub use data_access::*;
#[cfg(not(target_arch = "wasm32"))]
pub use guard::*;
#[cfg(not(target_arch = "wasm32"))]
pub use program::*;
#[cfg(not(target_arch = "wasm32"))]
pub use remaining::*;
#[cfg(not(target_arch = "wasm32"))]
pub use signer::*;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
pub use solana_program;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
pub use solana_program::account_info::AccountInfo;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
pub use solana_program::msg;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
pub use solana_program::program_error::ProgramError;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend", not(feature = "pinocchio")))]
pub use solana_program::pubkey::Pubkey;

#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
pub use pinocchio;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
pub use pinocchio_backend::{AccountInfo, ProgramError, Pubkey};
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
pub use solana_program_log::log as msg;

#[cfg(target_arch = "wasm32")]
pub mod msg {
    #[macro_export]
    macro_rules! msg {
        ($($arg:tt)*) => {};
    }
    pub use msg;
}

#[cfg(target_arch = "wasm32")]
pub use solana_program_error::ProgramError;

#[cfg(target_arch = "wasm32")]
pub mod solana_program {
    pub mod pubkey {
        pub use solana_pubkey::Pubkey;
    }
}

#[cfg(target_arch = "wasm32")]
pub use solana_program::pubkey::Pubkey;

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct Account<'a, T = ()>(std::marker::PhantomData<&'a T>);

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct Signer<'a>(std::marker::PhantomData<&'a ()>);

#[cfg(target_arch = "wasm32")]
impl<'a> Signer<'a> {
    pub fn address(&self) -> Pubkey {
        Pubkey::new_from_array([0; 32])
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct Program<'a>(std::marker::PhantomData<&'a ()>);

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone)]
pub struct Remaining<'a>(std::marker::PhantomData<&'a ()>);

// `Pubkey` is `solana_address::Address` on every backend and target, so the derivation is its own.
pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, program_id)
}

pub fn create_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Result<Pubkey, ()> {
    Pubkey::create_program_address(seeds, program_id).map_err(|_| ())
}

pub trait PubkeyExt {
    fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8);
    fn create_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Result<Pubkey, ()>;
}

impl PubkeyExt for Pubkey {
    fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        find_program_address(seeds, program_id)
    }

    fn create_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Result<Pubkey, ()> {
        create_program_address(seeds, program_id)
    }
}

pub use solarium::*;
