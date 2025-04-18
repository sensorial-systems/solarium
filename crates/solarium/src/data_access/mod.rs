use crate::{prelude::*, Guard};
use borsh::{BorshSerialize, BorshDeserialize};

pub trait DataAccess<'a, T: BorshSerialize + BorshDeserialize> {
    fn data(self) -> Result<Guard<'a, T>>;
}
