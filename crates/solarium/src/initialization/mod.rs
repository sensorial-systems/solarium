use crate::prelude::*;

/// An account this program creates.
///
/// Discriminated rather than merely serializable: the bytes it writes have to start with the tag
/// that says which account they are, or a later read of the program's accounts cannot tell them
/// from another type's.
pub trait Initialization: Owner + Discriminator {}
