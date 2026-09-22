use crate::{prelude::*, Account, AccountInfo, Program, ProgramError, Signer};
use solana_program_log::log as msg;

pub struct Guard<'a, T> {
    pub account: AccountInfo<'a>,
    pub data: T,
}

impl<T> core::ops::Deref for Guard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// An account's data, decoded for editing and written back when the guard goes.
///
/// A write that cannot be made in full is never made in part. `commit` reports the failure; a
/// guard simply dropped has nobody to report it to, so it aborts the instruction instead — the
/// runtime then discards every change the instruction made, which is the only outcome safe for an
/// edit that did not land.
pub struct GuardMut<'a, T: Discriminator> {
    pub account: AccountInfo<'a>,
    pub resize: Option<(Signer<'a>, Program<'a>)>,
    pub data: T,
    written: bool,
}

impl<'a, T: Discriminator> GuardMut<'a, T> {
    pub fn new(account: AccountInfo<'a>, data: T, resize: Option<(Signer<'a>, Program<'a>)>) -> Self {
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

    fn write_back(&mut self) -> Result<()> {
        let serialized = self.data.to_account_bytes()?;
        if let Some((signer, program)) = self.resize {
            Account::<'a, T>::account_realloc_to(
                &mut self.account,
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
        if self.write_back().is_err() {
            msg!("Account data could not be written back");
            panic!("account data could not be written back");
        }
    }
}

impl<T: Discriminator> core::ops::Deref for GuardMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Discriminator> core::ops::DerefMut for GuardMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pinocchio::account::{RuntimeAccount, NOT_BORROWED};

    #[solarium::account]
    #[derive(Debug, Default, PartialEq)]
    struct Note {
        text: String,
    }

    fn note_bytes(text: &str) -> Vec<u8> {
        Note { text: text.into() }.to_account_bytes().unwrap()
    }

    /// An account holding `stored` in `room` bytes, laid out as the runtime lays it out: the
    /// header, then the data. Leaked, as the runtime's accounts outlive the instruction.
    fn view(room: usize, stored: &str) -> &'static mut pinocchio::AccountView {
        let header = core::mem::size_of::<RuntimeAccount>();
        let words = (header + room).div_ceil(8);
        let memory: &'static mut [u64] = Box::leak(vec![0u64; words].into_boxed_slice());
        let raw = memory.as_mut_ptr() as *mut RuntimeAccount;
        unsafe {
            raw.write(RuntimeAccount {
                borrow_state: NOT_BORROWED,
                is_writable: 1,
                data_len: room as u64,
                ..Default::default()
            });
            let data = (raw as *mut u8).add(header);
            let initial = note_bytes(stored);
            assert!(initial.len() <= room, "the fixture does not fit its own account");
            core::ptr::copy_nonoverlapping(initial.as_ptr(), data, initial.len());
        }
        Box::leak(Box::new(unsafe { pinocchio::AccountView::new_unchecked(raw) }))
    }

    fn guard(view: &'static mut pinocchio::AccountView) -> GuardMut<'static, Note> {
        let info = AccountInfo::new(view);
        let data = Account::<Note>::new(info).deserialize().unwrap();
        GuardMut::new(info, data, None)
    }

    fn stored(view: &pinocchio::AccountView) -> Vec<u8> {
        view.try_borrow().unwrap().to_vec()
    }

    #[test]
    fn an_edit_that_fits_is_written_with_its_tag() {
        let view = view(64, "hi");
        let seen: *const pinocchio::AccountView = view;
        let mut note = guard(view);
        note.text = "hello".into();
        note.commit().unwrap();
        let expected = note_bytes("hello");
        assert_eq!(&stored(unsafe { &*seen })[..expected.len()], &expected[..]);
    }

    #[test]
    fn an_edit_that_does_not_fit_is_refused_whole() {
        let view = view(note_bytes("hi").len(), "hi");
        let seen: *const pinocchio::AccountView = view;
        let mut note = guard(view);
        note.text = "a note longer than the account".into();
        assert!(matches!(
            note.commit(),
            Err(Error::ProgramError(ProgramError::AccountDataTooSmall))
        ));
        // Not a truncated prefix of the new note: the old one, untouched.
        assert_eq!(stored(unsafe { &*seen }), note_bytes("hi"));
    }

    #[test]
    #[should_panic(expected = "account data could not be written back")]
    fn a_dropped_edit_that_does_not_fit_aborts() {
        let mut note = guard(view(note_bytes("hi").len(), "hi"));
        note.text = "a note longer than the account".into();
    }

    #[test]
    fn remaining_takes_every_account_left() {
        let views: &'static mut [pinocchio::AccountView] = Box::leak(
            (0..4)
                .map(|_| *view(note_bytes("").len(), ""))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        let accounts = &mut views.iter_mut();
        accounts.next();
        let rest = crate::Remaining::rest(accounts);
        assert_eq!(rest.len(), 3);
        assert!(accounts.next().is_none(), "the tail was left behind");
    }
}
