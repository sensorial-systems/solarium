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
        *self.view.address()
    }

    pub fn owner(&self) -> Pubkey {
        *self.view.owner()
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

/// The rent-exempt minimum for `space` bytes, as `solana-program`'s `Rent::minimum_balance` has
/// it: bytes times the byte-year rate, times the exemption threshold.
///
/// Pinocchio's own `Rent` reads the sysvar's rate as already including the threshold, which only
/// holds once SIMD-0194 has set the threshold to one; before that it quotes half, and an account
/// funded by it is not rent exempt.
pub(crate) fn minimum_balance(space: usize) -> Result<u64, ProgramError> {
    #[repr(C)]
    #[derive(Default)]
    struct Rent {
        lamports_per_byte_year: u64,
        exemption_threshold: f64,
        burn_percent: u8,
    }
    let mut rent = Rent::default();
    #[cfg(target_os = "solana")]
    {
        // The syscall `solana-program`'s `Rent::get` makes.
        #[allow(deprecated)]
        let result =
            unsafe { pinocchio::syscalls::sol_get_rent_sysvar(&mut rent as *mut Rent as *mut u8) };
        if result != 0 {
            return Err(ProgramError::UnsupportedSysvar);
        }
    }
    #[cfg(not(target_os = "solana"))]
    {
        rent.lamports_per_byte_year = 3480;
        rent.exemption_threshold = 2.0;
    }
    let _ = rent.burn_percent;
    let bytes = 128 + space as u64;
    Ok((bytes.saturating_mul(rent.lamports_per_byte_year) as f64 * rent.exemption_threshold) as u64)
}
