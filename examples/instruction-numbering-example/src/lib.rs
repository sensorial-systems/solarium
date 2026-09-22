//! A program that keeps a wire format it did not choose: its address is fixed, its instructions
//! are numbered, and one of them is also reached by a tag another program calls back with.

#![allow(unexpected_cfgs)]

use solarium::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use solarium_program::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use solarium_program::{Remaining, Signer};

#[program(id = "SLoTSdnmBH5KtNJjhEYw1MeWTKAfRnFfQTTcpgwRn2Q")]
impl Numbered {
    /// Answers to `"global:plain"`, as a method without the attribute always has.
    pub fn plain(&self, _payer: &Signer) -> Result<()> {
        Ok(())
    }

    #[instruction(discriminator = 16)]
    pub fn bet(&self, _payer: &Signer, machine: u64) -> Result<()> {
        let _ = machine;
        Ok(())
    }

    /// An oracle's answer: the randomness, then the round it was asked for.
    #[instruction(discriminator = 19)]
    pub fn reveal(&self, _oracle: &Signer, randomness: [u8; 32], round: u64) -> Result<()> {
        let _ = (randomness, round);
        Ok(())
    }

    /// Reports how many accounts followed the named one, so a test can see what it was given.
    #[instruction(discriminator = 11)]
    pub fn close(&self, _admin: &Signer, extra: &Remaining, which: u8) -> Result<()> {
        Err(ProgramError::Custom(u32::from(which) * 100 + extra.len() as u32).into())
    }

    #[instruction(discriminator = 3, alias = "global:process_undelegation")]
    pub fn undelegate(&self, _payer: &Signer, seeds: Vec<Vec<u8>>) -> Result<()> {
        let _ = seeds;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(discriminator: [u8; 8], arguments: &[u8]) -> Vec<u8> {
        [&discriminator[..], arguments].concat()
    }

    #[test]
    fn the_address_is_the_one_given() {
        assert_eq!(ID.to_string(), "SLoTSdnmBH5KtNJjhEYw1MeWTKAfRnFfQTTcpgwRn2Q");
    }

    #[test]
    fn a_numbered_method_answers_to_its_number() {
        assert_eq!(NumberedInstruction::BET, 16);
        let bet = Numbered::instruction(&input(16u64.to_le_bytes(), &7u64.to_le_bytes())).unwrap();
        assert!(matches!(bet, NumberedInstruction::Bet(NumberedBet { machine: 7 })));
    }

    #[test]
    fn an_alias_reaches_the_same_method() {
        let seeds = borsh::to_vec(&vec![b"house".to_vec()]).unwrap();
        let by_number = Numbered::instruction(&input(3u64.to_le_bytes(), &seeds)).unwrap();
        let by_tag = Numbered::instruction(&input(
            solarium::discriminator!("global:process_undelegation"),
            &seeds,
        ))
        .unwrap();
        assert!(matches!(by_number, NumberedInstruction::Undelegate(_)));
        assert!(matches!(by_tag, NumberedInstruction::Undelegate(_)));
        // What is sent is the method's own number, never the alias.
        assert_eq!(NumberedInstruction::UNDELEGATE, 3);
    }

    #[test]
    fn an_unnumbered_method_keeps_its_hashed_name() {
        let plain = Numbered::instruction(&solarium::discriminator!("global:plain")).unwrap();
        assert!(matches!(plain, NumberedInstruction::Plain(_)));
    }

    #[test]
    fn bytes_after_the_arguments_are_left_unread() {
        // A callback may append its own bytes; the method takes the arguments it declares.
        let bet = Numbered::instruction(&input(
            16u64.to_le_bytes(),
            &[&7u64.to_le_bytes()[..], &[0xAA; 40]].concat(),
        ))
        .unwrap();
        assert!(matches!(bet, NumberedInstruction::Bet(NumberedBet { machine: 7 })));
    }

    #[test]
    fn an_array_argument_is_read_whole() {
        let reveal = Numbered::instruction(&input(
            19u64.to_le_bytes(),
            &[&[9u8; 32][..], &4u64.to_le_bytes()].concat(),
        ))
        .unwrap();
        let NumberedInstruction::Reveal(reveal) = reveal else {
            panic!("not a reveal");
        };
        assert_eq!((reveal.randomness, reveal.round), ([9; 32], 4));
    }

    #[cfg(not(feature = "pinocchio"))]
    fn signer() -> solarium_program::AccountInfo<'static> {
        solarium_program::AccountInfo::new(
            Box::leak(Box::new(Pubkey::new_unique())),
            true,
            false,
            Box::leak(Box::new(0)),
            &mut [],
            Box::leak(Box::new(Pubkey::new_unique())),
            false,
        )
    }

    #[cfg(not(feature = "pinocchio"))]
    #[test]
    fn the_accounts_after_the_named_ones_are_handed_over() {
        let accounts = Box::leak(vec![signer(), signer(), signer(), signer()].into_boxed_slice());
        let close = input(11u64.to_le_bytes(), &[2]);
        let reported = process_instruction(&ID, accounts, &close).unwrap_err();
        assert_eq!(reported, ProgramError::Custom(203));
        let reported = process_instruction(&ID, &accounts[..1], &close).unwrap_err();
        assert_eq!(reported, ProgramError::Custom(200));
    }

    #[test]
    fn a_number_nothing_answers_to_is_refused() {
        assert!(Numbered::instruction(&input(17u64.to_le_bytes(), &[])).is_err());
        assert!(Numbered::instruction(&[16, 0, 0]).is_err());
    }
}
