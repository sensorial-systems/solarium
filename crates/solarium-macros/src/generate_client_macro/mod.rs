use anyhow::Result;
use ligen_rust_generator::{Generator, RustIdentifierGenerator};
use quote::quote;
use syn::{Ident, LitStr, Token};

struct Definition {
    literal: LitStr,
    _as_token: Token![as],
    identifier: Ident,
}

impl syn::parse::Parse for Definition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let literal = input.parse::<LitStr>()?;
        let _as_token = input.parse::<Token![as]>()?;
        let identifier = input.parse::<Ident>()?;
        Ok(Self { literal, _as_token, identifier })
    }
}

pub fn process(input: proc_macro::TokenStream) -> Result<proc_macro2::TokenStream> {
    let input = syn::parse::<Definition>(input)?;

    let client = ligen::ir::Identifier::new(&input.identifier.to_string());
    let message_builder = ligen::ir::Identifier::new(&format!("{}MessageBuilder", input.identifier));

    let generator = RustIdentifierGenerator::default();
    let client = generator.generate(&client, &Default::default())?;
    let message_builder = generator.generate(&message_builder, &Default::default())?;
    let program_name = &input.literal;

    let output = quote! {
        #[allow(unused_imports)]
        use solarium_client::prelude::*;

        pub struct #client {
            connection: solarium_client::Connection,
        }

        impl #client {
            pub fn new(connection: &solarium_client::Connection) -> Self {
                let connection = connection.clone();
                Self { connection }
            }        
        }

        impl solarium_client::Program for #client {
            type MessageBuilder = #message_builder;
        
            fn message_builder(&self) -> #message_builder {
                #message_builder::new(self.connection())
            }
        
            fn id() -> Pubkey {
                solarium_client::program_id!(#program_name)
            }
        
            fn connection(&self) -> &solarium_client::Connection {
                &self.connection
            }
        }

        pub struct #message_builder(solarium_client::MessageBuilder);

        impl std::ops::Deref for #message_builder {
            type Target = solarium_client::MessageBuilder;
        
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl std::ops::DerefMut for #message_builder {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl #message_builder {
            pub fn new(connection: &solarium_client::Connection) -> Self {
                Self(solarium_client::MessageBuilder::new(connection))
            }
        }
    };

    Ok(output)
}
