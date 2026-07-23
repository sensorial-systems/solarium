use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

pub fn process(input: &str) -> Result<TokenStream> {
    let discriminator = &Sha256::digest(input.as_bytes())[..8];
    Ok(quote! { [#(#discriminator),*] })
}
