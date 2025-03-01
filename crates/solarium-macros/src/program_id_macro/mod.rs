use ligen_ir::Identifier;
use proc_macro::TokenStream;
use quote::quote;
use solana_sdk::{signature::Keypair, signer::{EncodableKey, Signer}};
use anyhow::{Result, Context};

pub fn process(program_name: &str) -> Result<TokenStream> {
    let program_name = Identifier::from(program_name);
    let program_name = program_name.to_snake_case().to_string();
    let project_root = project_root::get_project_root().context("Failed to get project root")?;
    let keypair_path = project_root.join("target").join("deploy").join(format!("{}-keypair.json", program_name));
    let keypair = if keypair_path.exists() {
        Keypair::read_from_file(&keypair_path).map_err(|e| anyhow::anyhow!("Failed to load keypair: {}", e))?
    } else {
        let keypair = Keypair::new();
        keypair.write_to_file(&keypair_path).map_err(|e| anyhow::anyhow!("Failed to save keypair: {}", e))?;
        keypair
    };
    let pubkey = keypair.pubkey();
    let program_id = pubkey.to_bytes();

    let expanded = quote! {
        ::solana_program::pubkey::Pubkey::new_from_array([
            #(#program_id),*
        ])
    };

    Ok(TokenStream::from(expanded))
}