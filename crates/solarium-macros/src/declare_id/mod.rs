use proc_macro2::TokenStream;
use anyhow::Result;
use quote::quote;

pub fn process(input: TokenStream) -> Result<TokenStream> {
    let expanded = quote! {
        /// The const program ID.
        pub const ID: ::solarium::prelude::solana_program::pubkey::Pubkey = #input;
        /// Returns `true` if given pubkey is the program ID.
        pub fn check_id(id: &::solarium::prelude::solana_program::pubkey::Pubkey) -> bool {
            id == &ID
        }
        /// Returns the program ID.
        pub const fn id() -> ::solarium::prelude::solana_program::pubkey::Pubkey {
            ID
        }
    };
    Ok(expanded)
}
