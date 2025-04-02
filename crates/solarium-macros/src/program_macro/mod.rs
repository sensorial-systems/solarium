use anyhow::{Context, Result};
use ligen_anchor_generator::AnchorMethodGenerator;
use ligen_rust_parser::RustInterfaceParser;
use ligen_parser::Parser;
use ligen_generator::*;
use proc_macro2::TokenStream;
use quote::quote;
use solarium_workspace::Workspace;

pub fn process(input: syn::ItemImpl) -> Result<TokenStream> {
    let program_name = input.self_ty.clone();
    let original = input.clone();

    let parser = RustInterfaceParser::new();
    let input = parser.parse(input, &ligen_parser::ParserConfig::default()).expect("Failed to parse interface");

    let workspace = Workspace::current().context("Failed to get workspace")?;
    let program = std::env::var("CARGO_PKG_NAME").expect("Failed to get current program ID");
    let program = workspace.program(&program).context("Failed to get program")?;

    let mut idl = program.anchor_idl_from_file(&workspace).context("Failed to get program IDL")?;
    for method in input.methods.iter() {
        if method.identifier != "process_instruction" && !idl.instructions.iter().any(|t| t.name == method.identifier.to_string()) {
            let instruction = AnchorMethodGenerator::new().generate(method, &Default::default())?;
            idl.instructions.push(instruction);
        }
    }
    std::fs::write(&program.idl_path(&workspace), serde_json::to_string_pretty(&idl)?).context("Failed to write IDL")?;    

    Ok(quote! {
        struct #program_name;
        #original

        ::solarium::prelude::declare_id!(::solarium::current_program_id!());
        ::solarium::prelude::solana_program::entrypoint!(process_instruction);

        pub fn process_instruction(
            program_id: &solarium::prelude::solana_program::pubkey::Pubkey,      // Public key of the program
            accounts: &[solarium::prelude::solana_program::account_info::AccountInfo], // Data accounts, payer, etc.
            instruction_data: &[u8],  // External data passed to program
        ) -> solarium::prelude::solana_program::entrypoint::ProgramResult {
            #program_name.process_instruction(program_id, accounts, instruction_data)
        }
    })
}
