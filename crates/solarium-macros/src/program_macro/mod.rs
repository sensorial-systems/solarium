mod default_process_instruction;

use anyhow::Result;
use ligen::transformer::Transformer;
use ligen_rust::parser::RustInterfaceParser;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn process(input: syn::ItemImpl) -> Result<TokenStream> {
    let program_name = input.self_ty.clone();
    let mut program_impl = input.clone();

    let parser = RustInterfaceParser::new();
    let input = parser
        .transform(input, &ligen::transformer::Config::default())
        .expect("Failed to parse interface"); // FIXME: Remove expect

    let program_impl = if !input
        .methods
        .iter()
        .any(|m| m.identifier == "process_instruction")
    {
        default_process_instruction::generate(&mut program_impl, &input)?
    } else {
        program_impl.to_token_stream()
    };

    Ok(quote! {
        ::solarium::prelude::declare_id!(::solarium::current_program_id!());

        #program_impl

        #[cfg(all(
            not(target_arch = "wasm32"),
            not(feature = "no-entrypoint"),
            not(feature = "pinocchio")
        ))]
        solarium_program::prelude::solana_program::entrypoint!(process_instruction);

        #[cfg(all(not(target_arch = "wasm32"), not(feature = "pinocchio")))]
        pub fn process_instruction<'a>(
            program_id: &solarium_program::prelude::solana_program::pubkey::Pubkey,
            accounts: &'a [solarium_program::prelude::solana_program::account_info::AccountInfo<'a>],
            instruction_data: &[u8],
        ) -> solarium_program::prelude::solana_program::entrypoint::ProgramResult {
            let program = #program_name;
            Ok(program.process_instruction(program_id, accounts, instruction_data)?)
        }

        #[cfg(all(
            not(target_arch = "wasm32"),
            not(feature = "no-entrypoint"),
            feature = "pinocchio"
        ))]
        solarium_program::prelude::pinocchio::entrypoint!(process_instruction);

        #[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
        pub fn process_instruction(
            program_id: &solarium_program::prelude::pinocchio::Address,
            accounts: &mut [solarium_program::prelude::pinocchio::AccountView],
            instruction_data: &[u8],
        ) -> solarium_program::prelude::pinocchio::ProgramResult {
            let program = #program_name;
            program.process_instruction(program_id, accounts, instruction_data)
                .map_err(Into::into)
        }
    })
}
