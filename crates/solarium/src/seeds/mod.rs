pub trait Seeds {
    fn seeds(&self) -> Vec<Vec<u8>>;
}