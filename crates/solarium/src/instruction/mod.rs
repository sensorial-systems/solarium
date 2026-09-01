use crate::prelude::borsh::{BorshDeserialize, BorshSerialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct Instruction<Data> {
    pub discriminator: [u8; 8],
    pub data: Data,
}

impl<Data> Instruction<Data> {
    pub fn new(discriminator: [u8; 8], data: Data) -> Self {
        Self {
            discriminator,
            data,
        }
    }

    pub fn discriminator_for(name: impl AsRef<str>) -> [u8; 8] {
        let name = name.as_ref();
        let mut discriminator = [0u8; 8];
        let hash = Sha256::digest(name);
        discriminator.copy_from_slice(&hash[..8]);
        discriminator
    }
}
