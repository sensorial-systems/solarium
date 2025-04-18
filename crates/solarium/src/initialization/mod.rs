use crate::prelude::*;

pub trait Initialization: Default + Space + Pda + Owner + BorshSerialize {}