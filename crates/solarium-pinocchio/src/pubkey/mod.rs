
pub use pinocchio::pubkey::Pubkey as PinocchioPubkey;

pub struct Pubkey(pub pinocchio::pubkey::Pubkey);

impl core::ops::Deref for Pubkey {
    type Target = pinocchio::pubkey::Pubkey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<pinocchio::pubkey::Pubkey> for Pubkey {
    fn from(pubkey: pinocchio::pubkey::Pubkey) -> Self {
        Pubkey(pubkey)
    }
}

impl From<Pubkey> for pinocchio::pubkey::Pubkey {
    fn from(pubkey: Pubkey) -> Self {
        pubkey.0
    }
}

impl Pubkey {
    pub fn find_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
        let (pubkey, bump) = pinocchio::pubkey::find_program_address(seeds, &program_id.0);
        (pubkey.into(), bump)
    }

    pub const fn new_from_array(array: [u8; 32]) -> Self {
        Self(array)
    }
}

impl PartialEq<[u8; 32]> for Pubkey {
    fn eq(&self, other: &[u8; 32]) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&[u8; 32]> for Pubkey {
    fn eq(&self, other: &&[u8; 32]) -> bool {
        self.0 == **other
    }
}

impl PartialEq<Pubkey> for [u8; 32] {
    fn eq(&self, other: &Pubkey) -> bool {
        *self == other.0
    }
}

impl PartialEq<Pubkey> for &[u8; 32] {
    fn eq(&self, other: &Pubkey) -> bool {
        **self == other.0
    }
}

impl PartialEq<&Pubkey> for Pubkey {
    fn eq(&self, other: &&Pubkey) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<Pubkey> for &Pubkey {
    fn eq(&self, other: &Pubkey) -> bool {
        self.0 == other.0
    }
}


impl PartialEq for Pubkey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Pubkey {}
