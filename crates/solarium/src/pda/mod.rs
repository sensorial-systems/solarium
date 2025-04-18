pub trait Pda {
    fn seeds() -> &'static [&'static [u8]];
}
