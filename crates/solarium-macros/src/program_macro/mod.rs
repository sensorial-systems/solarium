use proc_macro2::TokenStream;
use quote::quote;

pub fn process(input: syn::ItemImpl) -> TokenStream {
    let program_name = &input.self_ty;

    quote! {
        struct #program_name;
        #input

        ::solarium::prelude::declare_id!(::solarium::current_program_id!());
        ::solarium::prelude::solana_program::entrypoint!(process_instruction);

        pub fn process_instruction(
            program_id: &solarium::prelude::solana_program::pubkey::Pubkey,      // Public key of the program
            accounts: &[solarium::prelude::solana_program::account_info::AccountInfo], // Data accounts, payer, etc.
            instruction_data: &[u8],  // External data passed to program
        ) -> solarium::prelude::solana_program::entrypoint::ProgramResult {
            #program_name.process_instruction(program_id, accounts, instruction_data)
        }
    }
}
