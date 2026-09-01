use core::marker::PhantomData;

pub use pinocchio::error::ProgramError;
pub use solarium::prelude::Pubkey;

#[derive(Clone, Copy)]
pub struct AccountInfo<'a> {
    view: pinocchio::AccountView,
    marker: PhantomData<&'a mut pinocchio::AccountView>,
}

impl<'a> AccountInfo<'a> {
    pub(crate) fn new(view: &'a mut pinocchio::AccountView) -> Self {
        Self {
            view: view.clone(),
            marker: PhantomData,
        }
    }

    pub fn key(&self) -> Pubkey {
        Pubkey::new_from_array(self.view.address().to_bytes())
    }

    pub fn owner(&self) -> Pubkey {
        Pubkey::new_from_array(self.view.owner().to_bytes())
    }

    pub fn is_signer(&self) -> bool {
        self.view.is_signer()
    }

    pub fn is_writable(&self) -> bool {
        self.view.is_writable()
    }

    pub fn executable(&self) -> bool {
        self.view.executable()
    }

    pub fn lamports(&self) -> u64 {
        self.view.lamports()
    }

    pub fn set_lamports(&self, lamports: u64) {
        let mut view = self.view;
        view.set_lamports(lamports);
    }

    pub fn data_len(&self) -> usize {
        self.view.data_len()
    }

    pub fn try_borrow_data(&self) -> Result<pinocchio::account::Ref<'_, [u8]>, ProgramError> {
        self.view.try_borrow()
    }

    pub fn try_borrow_mut_data(
        &mut self,
    ) -> Result<pinocchio::account::RefMut<'_, [u8]>, ProgramError> {
        self.view.try_borrow_mut()
    }

    pub fn as_view(&self) -> &pinocchio::AccountView {
        &self.view
    }

    pub(crate) fn view(&self) -> &pinocchio::AccountView {
        self.as_view()
    }

    pub(crate) fn view_mut(&mut self) -> &mut pinocchio::AccountView {
        &mut self.view
    }
}

pub(crate) fn address(pubkey: &Pubkey) -> pinocchio::Address {
    pinocchio::Address::new_from_array(pubkey.to_bytes())
}

pub(crate) fn pubkey(address: &pinocchio::Address) -> Pubkey {
    Pubkey::new_from_array(address.to_bytes())
}
