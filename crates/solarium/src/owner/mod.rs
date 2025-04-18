use solana_program::pubkey::Pubkey;

pub trait Owner {
    fn owner() -> &'static Pubkey;
}