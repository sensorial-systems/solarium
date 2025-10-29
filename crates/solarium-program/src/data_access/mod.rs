use crate::prelude::*;
use borsh::{BorshSerialize, BorshDeserialize};
use crate::{Program, Signer};

pub trait DataAccess<'a, T: BorshSerialize + BorshDeserialize> {
    type Output;
    fn data(self) -> Self::Output;
}

pub trait ResizableDataAccess<'a, T: BorshSerialize + BorshDeserialize> {
    type Output;
    fn resizeable_data(self, payer: &Signer<'a>, program: &Program<'a>) -> Self::Output;
}
