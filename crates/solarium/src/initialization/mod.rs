use crate::prelude::*;

pub trait Initialization: Default + Space + Owner + BorshSerialize {}