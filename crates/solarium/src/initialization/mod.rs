use crate::prelude::*;

pub trait Initialization: Default + Owner + BorshSerialize {}