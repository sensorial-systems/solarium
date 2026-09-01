use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub trait AccountInfoExt {
    fn key(&self) -> Pubkey;
    fn owner(&self) -> Pubkey;
    fn is_signer(&self) -> bool;
    fn is_writable(&self) -> bool;
    fn executable(&self) -> bool;
    fn set_lamports(&self, lamports: u64);
}

impl AccountInfoExt for AccountInfo<'_> {
    #[inline]
    fn key(&self) -> Pubkey {
        *self.key
    }

    #[inline]
    fn owner(&self) -> Pubkey {
        *self.owner
    }

    #[inline]
    fn is_signer(&self) -> bool {
        self.is_signer
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.is_writable
    }

    #[inline]
    fn executable(&self) -> bool {
        self.executable
    }

    #[inline]
    fn set_lamports(&self, lamports: u64) {
        **self.lamports.borrow_mut() = lamports;
    }
}
