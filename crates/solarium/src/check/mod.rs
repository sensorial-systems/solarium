use crate::prelude::*;

pub trait Check {
    fn check(&self) -> Result<()>;
}
