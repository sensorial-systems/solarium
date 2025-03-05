use proc_macro::TokenStream;
use quote::quote;
use solarium_workspace::Workspace;
use anyhow::Result;
pub fn process(program_name: &str) -> Result<TokenStream> {
    let workspace = Workspace::current()?;
    let program = workspace.programs.iter().find(|p| p.name == program_name).ok_or(anyhow::anyhow!("Program {} not found", program_name))?;
    let program_id = program.public_key;
    let program_id = program_id.to_bytes();

    let expanded = quote! {
        ::solana_program::pubkey::Pubkey::new_from_array([
            #(#program_id),*
        ])
    };

    Ok(TokenStream::from(expanded))
}