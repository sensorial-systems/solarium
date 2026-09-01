pub mod prelude;

pub mod result;

#[cfg(all(feature = "pinocchio", feature = "solana-program-backend"))]
compile_error!(r#"features "pinocchio" and "solana-program-backend" are mutually exclusive"#);
#[cfg(all(
    not(target_arch = "wasm32"),
    not(feature = "pinocchio"),
    not(feature = "solana-program-backend")
))]
compile_error!(r#"select either the "solana-program-backend" or "pinocchio" feature"#);

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod account;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/account.rs"]
mod account;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod account_info;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod account_initialization;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/account_initialization.rs"]
mod account_initialization;

#[cfg(not(target_arch = "wasm32"))]
mod check;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod context;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/context.rs"]
mod context;

#[cfg(not(target_arch = "wasm32"))]
mod data_access;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod guard;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/guard.rs"]
mod guard;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod program;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/program.rs"]
mod program;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod signer;
#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/signer.rs"]
mod signer;

#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
mod system_instruction;

#[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
#[path = "pinocchio/mod.rs"]
mod pinocchio_backend;

#[cfg(not(target_arch = "wasm32"))]
pub use account::*;
#[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
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
pub use signer::*;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramError {
    Custom(u32),
    InvalidArgument,
    InvalidInstructionData,
    InvalidAccountData,
    AccountDataTooSmall,
    InsufficientFunds,
    IncorrectProgramId,
    MissingRequiredSignature,
    AccountAlreadyInitialized,
    UninitializedAccount,
    NotEnoughAccountKeys,
    AccountBorrowFailed,
    MaxSeedLengthExceeded,
    InvalidSeeds,
    BorshIoError,
    AccountNotRentExempt,
    UnsupportedSysvar,
    IllegalOwner,
    MaxAccountsDataAllocationsExceeded,
    InvalidRealloc,
    ComputationalBudgetExceeded,
    PrivilegeEscalation,
    ProgramEnvironmentSetupFailure,
    ProgramFailedToComplete,
    ProgramFailedToCompile,
    Immutable,
    IncorrectAuthority,
    AccountNotExecutable,
}

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

pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
    {
        solana_program::pubkey::Pubkey::find_program_address(seeds, program_id)
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
    {
        let program_id = pinocchio_backend::address(program_id);
        let (address, bump) = solana_address::Address::find_program_address(seeds, &program_id);
        (pinocchio_backend::pubkey(&address), bump)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut bump_seed = [u8::MAX];
        for bump in (0..=u8::MAX).rev() {
            bump_seed[0] = bump;
            let mut seeds_with_bump = seeds.to_vec();
            seeds_with_bump.push(&bump_seed);
            if let Ok(address) = create_program_address(&seeds_with_bump, program_id) {
                return (address, bump);
            }
        }
        panic!("Could not find a valid program address");
    }
}

pub fn create_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> Result<Pubkey, ()> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "solana-program-backend"))]
    {
        solana_program::pubkey::Pubkey::create_program_address(seeds, program_id).map_err(|_| ())
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
    {
        let program_id = pinocchio_backend::address(program_id);
        solana_address::Address::create_program_address(seeds, &program_id)
            .map(|address| pinocchio_backend::pubkey(&address))
            .map_err(|_| ())
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut hasher = solana_sha256_hasher::Hasher::default();
        for seed in seeds {
            hasher.hash(seed);
        }
        hasher.hash(program_id.as_ref());
        hasher.hash(b"ProgramDerivedAddress");
        let hash = hasher.result();
        let bytes: [u8; 32] = hash.to_bytes();
        if curve25519_dalek::edwards::CompressedEdwardsY(bytes)
            .decompress()
            .is_some()
        {
            Err(())
        } else {
            Ok(Pubkey::new_from_array(bytes))
        }
    }
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
