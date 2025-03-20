use proc_macro::TokenStream;
use quote::quote;
use solarium_workspace::{Program, Workspace};
use anyhow::{Context, Result};

pub fn process(program_name: &str) -> Result<TokenStream> {
    let workspace = Workspace::current()?;
    let program_id = Program::get_program_id_from_file(&workspace, program_name).context("Failed to fetch program ID")?;
    let program_id = program_id.to_bytes();

    let expanded = quote! {
        ::solarium::prelude::solana_program::pubkey::Pubkey::new_from_array([
            #(#program_id),*
        ])
    };

    Ok(TokenStream::from(expanded))
}