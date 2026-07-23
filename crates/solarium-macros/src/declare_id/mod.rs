use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

pub fn process(input: TokenStream) -> Result<TokenStream> {
    let expanded = quote! {
        /// The const program ID.
        pub const ID: solarium_program::prelude::solana_program::pubkey::Pubkey = #input;
        /// Returns `true` if given pubkey is the program ID.
        pub fn check_id(id: &solarium_program::prelude::solana_program::pubkey::Pubkey) -> bool {
            id == &ID
        }
        /// Returns the program ID.
        pub const fn id() -> solarium_program::prelude::solana_program::pubkey::Pubkey {
            ID
        }
    };
    Ok(expanded)
}
