use anyhow::Result;
use ligen::idl::Attributes;
use proc_macro2::TokenStream;
use quote::quote;

pub fn process(input: syn::ItemStruct, attributes: Attributes) -> Result<TokenStream> {
    let struct_name = &input.ident;

    // Every account carries a tag saying which type it is, because nothing else about its bytes
    // does: a program owns several kinds of account, and Borsh reads whichever of them happen to
    // fit the shape it was asked for. Namespaced so an account and an instruction sharing a name
    // cannot share a tag, and taken from the type's own name so two account types never can.
    let name = syn::LitStr::new(
        &format!("account:{struct_name}"),
        proc_macro2::Span::call_site(),
    );
    let discriminator = quote! {
        impl solarium::Discriminator for #struct_name {
            const DISCRIMINATOR: [u8; solarium::DISCRIMINATOR_LEN] =
                solarium::discriminator!(#name);
        }
    };

    let pda = attributes
        .get_group("pda")
        .map(|_pda| {
            quote! {
                impl solarium::Owner for #struct_name {
                    fn owner() -> &'static solarium::prelude::Pubkey {
                        &crate::ID
                    }
                }

                impl solarium::Initialization for #struct_name {}
            }
        })
        .unwrap_or_default();

    Ok(quote! {
        #[derive(solarium::prelude::borsh::BorshSerialize, solarium::prelude::borsh::BorshDeserialize)]
        #[borsh(crate = "solarium::prelude::borsh")]
        #input

        #discriminator

        #pda
    })
}
