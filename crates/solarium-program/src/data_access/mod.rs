use crate::prelude::*;
use borsh::{BorshSerialize, BorshDeserialize};

pub trait DataAccess<'a, T: BorshSerialize + BorshDeserialize> {
    type Output;
    fn data(self) -> Self::Output;
}
