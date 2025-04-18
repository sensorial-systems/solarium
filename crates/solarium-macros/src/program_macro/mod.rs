mod default_process_instruction;

use anyhow::Result;
use ligen_rust_parser::RustInterfaceParser;
use ligen_parser::Parser;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn process(input: syn::ItemImpl) -> Result<TokenStream> {
    let program_name = input.self_ty.clone();
    let mut program_impl = input.clone();

    let parser = RustInterfaceParser::new();
    let input = parser.parse(input, &ligen_parser::ParserConfig::default()).expect("Failed to parse interface"); // FIXME: Remove expect

    let program_impl = if !input.methods.iter().any(|m| m.identifier == "process_instruction") {
        default_process_instruction::generate(&mut program_impl, &input)?
    } else {
        program_impl.to_token_stream()
    };

    Ok(quote! {
        #program_impl

        ::solarium::prelude::declare_id!(::solarium::current_program_id!());
        ::solarium::prelude::solana_program::entrypoint!(process_instruction);

        pub fn process_instruction<'a>(
            program_id: &solarium::prelude::solana_program::pubkey::Pubkey, // Public key of the program
            accounts: &'a [solarium::prelude::solana_program::account_info::AccountInfo<'a>], // Data accounts, payer, etc.
            instruction_data: &[u8],  // External data passed to program
        ) -> solarium::prelude::solana_program::entrypoint::ProgramResult {
            let program = #program_name;
            program.process_instruction(program_id, accounts, instruction_data)
        }
    })
}
