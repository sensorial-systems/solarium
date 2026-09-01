use crate::{prelude::*, AccountInfo};

#[derive(Clone, Copy)]
pub struct Program<'a> {
    pub info: AccountInfo<'a>,
}

impl<'a> TryFrom<&'a mut pinocchio::AccountView> for Program<'a> {
    type Error = Error;

    fn try_from(view: &'a mut pinocchio::AccountView) -> Result<Self> {
        let info = AccountInfo::new(view);
        if info.executable() {
            Ok(Self { info })
        } else {
            Err(ProgramError::InvalidAccountData.into())
        }
    }
}
