use ligen::idl::Attributes;
use proc_macro2::TokenStream;
use quote::quote;
use anyhow::Result;

pub fn process(input: syn::ItemStruct, attributes: Attributes) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let pda = attributes.get_group("pda").map(|pda| {
        let space = pda.get_named("space").map(|space| {
            let space = *space.as_integer().expect("Space must be an integer") as usize;
            quote! {
                impl solarium::Space for #struct_name {
                    fn space() -> usize {
                        #space
                    }
                }
            }
        }).unwrap_or_default();

        quote! {
            impl solarium::Owner for #struct_name {
                fn owner() -> &'static solarium::prelude::Pubkey {
                    &ID
                }
            }

            impl solarium::Initialization for #struct_name {}

            #space
        }
    }).unwrap_or_default();

    Ok(quote! {
        #[derive(solarium::prelude::borsh::BorshSerialize, solarium::prelude::borsh::BorshDeserialize)]
        #[borsh(crate = "solarium::prelude::borsh")]
        #input

        #pda
    })
}
