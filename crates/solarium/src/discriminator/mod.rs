use crate::prelude::*;

/// How many bytes at the front of an account say what kind of account it is.
///
/// Eight, matching the tag an instruction carries, so both halves of the protocol are read the
/// same way.
pub const DISCRIMINATOR_LEN: usize = 8;

/// The tag written ahead of an account's data.
///
/// A program owns more than one kind of account, and nothing about the bytes says which is which:
/// Borsh reads whatever it is given against the shape it was asked for, and succeeds whenever the
/// bytes happen to fit. A counter whose first four bytes pass for a string length becomes a name;
/// a scan of the program's accounts then counts it as one. The tag is derived from the type's own
/// name, so no account of one type can be read as another however its bytes fall.
///
/// Implemented by `#[account]`. Writing it by hand would let two types share a tag, which is the
/// one thing it exists to prevent.
pub trait Discriminator: BorshSerialize + BorshDeserialize + Sized {
    const DISCRIMINATOR: [u8; DISCRIMINATOR_LEN];

    /// Whether these bytes are an account of this type. Cheap: it reads eight bytes and no more,
    /// which is what makes it usable as a filter over a whole program's accounts.
    fn is_discriminated(bytes: &[u8]) -> bool {
        bytes
            .get(..DISCRIMINATOR_LEN)
            .is_some_and(|tag| tag == Self::DISCRIMINATOR)
    }

    /// The account's bytes: its tag, then its data.
    fn to_account_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(DISCRIMINATOR_LEN);
        bytes.extend_from_slice(&Self::DISCRIMINATOR);
        self.serialize(&mut bytes)?;
        Ok(bytes)
    }

    /// Reads an account of this type, refusing bytes that carry a different tag.
    ///
    /// Deserializes rather than reading exactly: an account can carry trailing bytes it has not
    /// been resized out of yet, and what follows the data is not the data's business.
    fn from_account_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        if !Self::is_discriminated(bytes) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "the account is not of this type",
            ));
        }
        Self::deserialize(&mut &bytes[DISCRIMINATOR_LEN..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
    struct Domain {
        name: String,
        owner: [u8; 32],
        parent: [u8; 32],
        price: Option<u64>,
    }

    /// Ten counters, and no field a domain would recognise.
    #[derive(BorshSerialize, BorshDeserialize, Debug, Default, PartialEq)]
    struct Stats {
        counters: [u64; 10],
    }

    impl Discriminator for Domain {
        const DISCRIMINATOR: [u8; DISCRIMINATOR_LEN] = crate::discriminator!("account:Domain");
    }

    impl Discriminator for Stats {
        const DISCRIMINATOR: [u8; DISCRIMINATOR_LEN] = crate::discriminator!("account:Stats");
    }

    #[test]
    fn an_account_reads_back_as_itself() {
        let domain = Domain {
            name: "example.chain".into(),
            owner: [1; 32],
            parent: [0; 32],
            price: Some(20_000_000),
        };
        let bytes = domain.to_account_bytes().unwrap();
        assert_eq!(&bytes[..DISCRIMINATOR_LEN], &Domain::DISCRIMINATOR);
        assert_eq!(Domain::from_account_bytes(&bytes).unwrap(), domain);
    }

    #[test]
    fn two_account_types_are_never_read_for_each_other() {
        // The whole point. A `Stats` is eighty bytes of numbers, which without a tag Borsh reads
        // as a `Domain` quite happily: the first four bytes become a name length and the rest
        // falls into place.
        let stats = Stats {
            counters: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        let untagged = borsh::to_vec(&stats).unwrap();
        assert!(
            Domain::deserialize(&mut &untagged[..]).is_ok(),
            "without a tag, a pile of counters passes for a name"
        );

        let tagged = stats.to_account_bytes().unwrap();
        assert!(
            Domain::from_account_bytes(&tagged).is_err(),
            "with one, it does not"
        );
        assert_eq!(Stats::from_account_bytes(&tagged).unwrap(), stats);
    }

    #[test]
    fn the_tag_is_the_types_own() {
        assert_ne!(Domain::DISCRIMINATOR, Stats::DISCRIMINATOR);
        assert_eq!(Domain::DISCRIMINATOR.len(), DISCRIMINATOR_LEN);
    }

    #[test]
    fn a_short_or_empty_account_is_not_one_of_ours() {
        // A freshly created account is zeroed, and zeroes are not a tag.
        assert!(!Domain::is_discriminated(&[]));
        assert!(!Domain::is_discriminated(&[0; DISCRIMINATOR_LEN]));
        assert!(!Domain::is_discriminated(&Domain::DISCRIMINATOR[..4]));
        assert!(Domain::from_account_bytes(&[0; 64]).is_err());
    }

    #[test]
    fn data_written_after_the_tag_survives_trailing_bytes() {
        // An account keeps whatever length it was last resized to, so a read has to tolerate bytes
        // beyond the data without mistaking them for more of it.
        let domain = Domain {
            name: "example.chain".into(),
            ..Default::default()
        };
        let mut bytes = domain.to_account_bytes().unwrap();
        bytes.extend_from_slice(&[0; 32]);
        assert_eq!(Domain::from_account_bytes(&bytes).unwrap(), domain);
    }
}
