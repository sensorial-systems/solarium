pub mod prelude;
pub mod result;

#[cfg(feature = "rpc")]
mod account;
#[cfg(feature = "rpc")]
mod connection;
#[cfg(feature = "rpc")]
mod message;
#[cfg(feature = "rpc")]
mod message_builder;
#[cfg(feature = "rpc")]
mod program;
#[cfg(feature = "rpc")]
mod sendable;
#[cfg(feature = "rpc")]
mod subscription;
#[cfg(feature = "rpc")]
mod transaction;
#[cfg(feature = "rpc")]
pub mod utils;

#[cfg(feature = "rpc")]
pub use account::Account;
#[cfg(feature = "rpc")]
pub use connection::Connection;
#[cfg(feature = "rpc")]
pub use message::Message;
#[cfg(feature = "rpc")]
pub use message_builder::MessageBuilder;
#[cfg(feature = "rpc")]
pub use program::Program;
#[cfg(feature = "rpc")]
pub use sendable::Sendable;
#[cfg(feature = "rpc")]
pub use subscription::Subscription;
#[cfg(feature = "rpc")]
pub use transaction::Transaction;

#[cfg(feature = "rpc")]
pub use solana_sdk::signature::Keypair;
pub use solarium::Instruction;

pub use solarium::*;

pub mod wire {
    pub use solana_instruction::*;
    pub use solana_pubkey::*;
}

/// What a generated client emits only when it can talk to a cluster: the client that holds a
/// connection, its message builder and the `Program` impl. The builders of each instruction are
/// emitted either way; these two macros keep the rest out of a build without `rpc`, where the
/// types they name do not exist. The feature is this crate's, so the consumer never declares one.
#[doc(hidden)]
#[cfg(feature = "rpc")]
#[macro_export]
macro_rules! __rpc {
    ($($item:tt)*) => { $($item)* };
}

#[doc(hidden)]
#[cfg(not(feature = "rpc"))]
#[macro_export]
macro_rules! __rpc {
    ($($item:tt)*) => {};
}

#[doc(hidden)]
#[cfg(feature = "rpc")]
#[macro_export]
macro_rules! __no_rpc {
    ($($item:tt)*) => {};
}

#[doc(hidden)]
#[cfg(not(feature = "rpc"))]
#[macro_export]
macro_rules! __no_rpc {
    ($($item:tt)*) => { $($item)* };
}
