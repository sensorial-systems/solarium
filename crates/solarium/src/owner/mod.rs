use crate::prelude::*;

pub trait Owner {
    fn owner() -> &'static Pubkey;
}