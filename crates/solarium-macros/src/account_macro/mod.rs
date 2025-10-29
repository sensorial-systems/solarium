use ligen::idl::Attributes;
use proc_macro2::TokenStream;
use quote::quote;
use anyhow::Result;

pub fn process(input: syn::ItemStruct, attributes: Attributes) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let pda = attributes.get_group("pda").map(|_pda| {
        quote! {
            impl solarium::Owner for #struct_name {
                fn owner() -> &'static solarium::prelude::Pubkey {
                    &crate::ID
                }
            }

            impl solarium::Initialization for #struct_name {}
        }
    }).unwrap_or_default();

    Ok(quote! {
        #[derive(solarium::prelude::borsh::BorshSerialize, solarium::prelude::borsh::BorshDeserialize)]
        #[borsh(crate = "solarium::prelude::borsh")]
        #input

        #pda
    })
}
