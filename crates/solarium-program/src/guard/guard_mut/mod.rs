use crate::{prelude::*, Account, Program, Signer};

use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;

/// An account's data, decoded for editing and written back when the guard goes.
///
/// A write that cannot be made in full is never made in part. `commit` reports the failure; a
/// guard simply dropped has nobody to report it to, so it aborts the instruction instead — the
/// runtime then discards every change the instruction made, which is the only outcome safe for an
/// edit that did not land.
pub struct GuardMut<'a, T: Discriminator> {
    pub account: &'a AccountInfo<'a>,
    pub resize: Option<(Signer<'a>, Program<'a>)>,
    pub data: T,
    written: bool,
}

impl<'a, T: Discriminator> GuardMut<'a, T> {
    pub fn new(account: &'a AccountInfo<'a>, data: T, resize: Option<(Signer<'a>, Program<'a>)>) -> Self {
        Self {
            account,
            resize,
            data,
            written: false,
        }
    }

    /// Writes the data back now, and says whether it could be.
    pub fn commit(mut self) -> Result<()> {
        self.written = true;
        self.write_back()
    }

    fn write_back(&self) -> Result<()> {
        // Written back with its tag, so the account still says what it is after an edit.
        let serialized = self.data.to_account_bytes()?;
        if let Some((signer, program)) = self.resize {
            Account::<'a, T>::account_realloc_to(
                self.account,
                &signer,
                &program,
                serialized.len(),
                false,
            )?;
        }
        let mut data = self.account.try_borrow_mut_data()?;
        let target = data
            .get_mut(..serialized.len())
            .ok_or(ProgramError::AccountDataTooSmall)?;
        target.copy_from_slice(&serialized);
        Ok(())
    }
}

impl<'a, T: Discriminator> Drop for GuardMut<'a, T> {
    fn drop(&mut self) {
        if self.written {
            return;
        }
        if let Err(error) = self.write_back() {
            msg!("Account data could not be written back: {}", error);
            panic!("account data could not be written back");
        }
    }
}

impl<'a, T: Discriminator> std::ops::Deref for GuardMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: Discriminator> std::ops::DerefMut for GuardMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[solarium::account]
    #[derive(Debug, Default, PartialEq)]
    struct Note {
        text: String,
    }

    fn note_bytes(text: &str) -> Vec<u8> {
        Note { text: text.into() }.to_account_bytes().unwrap()
    }

    /// An account holding `stored` in `room` bytes. Leaked, as the runtime's accounts outlive the
    /// instruction that borrows them.
    fn account(room: usize, stored: &str) -> &'static AccountInfo<'static> {
        let mut data = vec![0u8; room];
        let initial = note_bytes(stored);
        data[..initial.len()].copy_from_slice(&initial);
        Box::leak(Box::new(AccountInfo::new(
            Box::leak(Box::new(Pubkey::new_unique())),
            false,
            true,
            Box::leak(Box::new(0)),
            Box::leak(data.into_boxed_slice()),
            Box::leak(Box::new(Pubkey::new_unique())),
            false,
        )))
    }

    fn guard(info: &'static AccountInfo<'static>) -> GuardMut<'static, Note> {
        let data = Account::<Note>::new(info).deserialize().unwrap();
        GuardMut::new(info, data, None)
    }

    #[test]
    fn an_edit_that_fits_is_written_with_its_tag() {
        let info = account(64, "hi");
        let mut note = guard(info);
        note.text = "hello".into();
        note.commit().unwrap();
        let expected = note_bytes("hello");
        assert_eq!(&info.try_borrow_data().unwrap()[..expected.len()], &expected[..]);
    }

    #[test]
    fn an_edit_that_does_not_fit_is_refused_whole() {
        let info = account(note_bytes("hi").len(), "hi");
        let mut note = guard(info);
        note.text = "a note longer than the account".into();
        assert!(matches!(
            note.commit(),
            Err(Error::ProgramError(ProgramError::AccountDataTooSmall))
        ));
        // Not a truncated prefix of the new note: the old one, untouched.
        assert_eq!(&info.try_borrow_data().unwrap()[..], &note_bytes("hi")[..]);
    }

    #[test]
    #[should_panic(expected = "account data could not be written back")]
    fn a_dropped_edit_that_does_not_fit_aborts() {
        let info = account(note_bytes("hi").len(), "hi");
        let mut note = guard(info);
        note.text = "a note longer than the account".into();
    }
}
