use crate::prelude::*;

pub trait AccountUpdate {
    fn update(self) -> Result<()>;
}