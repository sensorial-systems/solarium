use ligen::common::anyhow::Context;
use ligen::generator::{Config, Generator};
use ligen::prelude::*;
use ligen_rust::generator::{RustIdentifierGenerator, RustLiteralGenerator, RustTypeGenerator};
use quote::quote;

#[derive(Default)]
pub struct ModuleGenerator {
    identifier_generator: RustIdentifierGenerator,
    literal_generator: RustLiteralGenerator,
    type_generator: RustTypeGenerator,
}

impl ModuleGenerator {
    pub fn generate_client_base(
        &self,
        program_id: &proc_macro2::TokenStream,
        client: &syn::Ident,
        message_builder: &syn::Ident,
    ) -> Result<proc_macro2::TokenStream> {
        Ok(quote! {
            #[allow(unused_imports)]
            use solarium_client::prelude::*;

            pub struct #client {
                connection: solarium_client::Connection,
                address: Pubkey,
            }


            impl #client {
                pub fn new(connection: &solarium_client::Connection) -> Self {
                    let connection = connection.clone();
                    let address = #program_id;
                    Self { connection, address }
                }

                pub fn with_address(mut self, address: impl Into<Pubkey>) -> Self {
                    self.address = address.into();
                    self
                }

                pub fn address(&self) -> Pubkey {
                    self.address
                }
            }

            impl solarium_client::Program for #client {
                type MessageBuilder = #message_builder;

                fn message_builder(&self) -> #message_builder {
                    #message_builder::new(self.connection(), self.address)
                }

                fn id() -> Pubkey {
                    #program_id
                }

                fn connection(&self) -> &solarium_client::Connection {
                    &self.connection
                }
            }

            pub struct #message_builder(solarium_client::MessageBuilder, Pubkey);

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
                pub fn new(connection: &solarium_client::Connection, address: Pubkey) -> Self {
                    Self(solarium_client::MessageBuilder::new(connection), address)
                }
            }
        })
    }
}

impl Generator<&ligen::idl::Module, syn::ItemMod> for ModuleGenerator {
    fn generate(&self, module: &ligen::idl::Module, config: &Config) -> Result<syn::ItemMod> {
        let ident = self
            .identifier_generator
            .generate(&module.identifier, config)?;
        let program_name = config
            .get("program-name")
            .context("Program name not found")?;
        let mut items: Vec<proc_macro2::TokenStream> = Default::default();
        for interface in &module.interfaces {
            if interface.attributes.get_group("program").is_some() {
                let client = &interface.identifier;
                let message_builder = client + "MessageBuilder";
                let client = self.identifier_generator.generate(&client, config)?;
                let program_name = self.literal_generator.generate(&program_name, config)?;
                let program_id = match config.get("program-address") {
                    Some(address) => {
                        let address = self.literal_generator.generate(&address, config)?;
                        quote! {
                            <solarium_client::wire::Pubkey as std::str::FromStr>::from_str(#address)
                                .expect("invalid custom Solana program address")
                        }
                    }
                    None => quote! {
                        solarium_client::program_id!(#program_name)
                    },
                };
                let program_crate_lit = config.get("program-crate").expect("program-crate not set");
                let program_crate: syn::Ident = syn::parse_str(
                    program_crate_lit
                        .as_string()
                        .as_deref()
                        .expect("program-crate must be string literal"),
                )
                .expect("invalid program-crate ident");
                let message_builder = self
                    .identifier_generator
                    .generate(&message_builder, config)?;

                let client_base =
                    self.generate_client_base(&program_id, &client, &message_builder)?;

                let mut client_methods = Vec::new();
                let mut message_builder_methods = Vec::new();
                for method in &interface.methods {
                    if method.visibility == ligen::idl::Visibility::Public {
                        let method_name = &method.identifier;
                        let method_name =
                            self.identifier_generator.generate(&method_name, config)?;
                        let instruction = &method.identifier + "_instruction";
                        let instruction =
                            self.identifier_generator.generate(&instruction, config)?;
                        let instruction_with_address =
                            &method.identifier + "_instruction_with_address";
                        let instruction_with_address = self
                            .identifier_generator
                            .generate(&instruction_with_address, config)?;
                        let discriminator =
                            ligen::idl::Literal::from(&format!("global:{}", method.identifier));
                        let discriminator =
                            self.literal_generator.generate(&discriminator, config)?;
                        let mut parameters: Vec<proc_macro2::TokenStream> = Vec::new();
                        let mut arguments: Vec<proc_macro2::TokenStream> = Vec::new();
                        let mut message_builder_arguments: Vec<proc_macro2::TokenStream> =
                            Vec::new();
                        let mut accounts = Vec::new();
                        for parameter in &method.inputs {
                            let name = &parameter.identifier;
                            let name = self.identifier_generator.generate(&name, config)?;
                            message_builder_arguments.push(quote! {
                                #name
                            });
                            if parameter.type_.is_mutable_reference()
                                || parameter.type_.is_constant_reference()
                            {
                                let is_writable = ligen::idl::Literal::from(
                                    parameter.type_.is_mutable_reference(),
                                );
                                let is_signer = ligen::idl::Literal::from(
                                    parameter.type_.path.last().identifier == "Signer",
                                );
                                let is_writable =
                                    self.literal_generator.generate(&is_writable, config)?;
                                let is_signer =
                                    self.literal_generator.generate(&is_signer, config)?;
                                accounts.push(quote! {
                                    solarium_client::wire::AccountMeta {
                                        is_signer: #is_signer,
                                        is_writable: #is_writable,
                                        pubkey: #name.into(),
                                    }
                                });
                                parameters.push(quote! {
                                    #name: impl Into<solarium_client::wire::Pubkey>
                                });
                            } else {
                                let type_ =
                                    self.type_generator.generate(&parameter.type_, config)?;
                                parameters.push(quote! {
                                    #name: impl Into<#type_>
                                });
                                arguments.push(quote! {
                                    #name.into()
                                });
                            }
                        }
                        if arguments.is_empty() {
                            arguments.push(quote! { () });
                        }
                        let client_method = quote! {
                            pub fn #instruction(#(#parameters),*) -> Result<solarium_client::wire::Instruction> {
                                Self::#instruction_with_address(#program_id, #(#message_builder_arguments),*)
                            }

                            fn #instruction_with_address(
                                program_address: solarium_client::wire::Pubkey,
                                #(#parameters),*
                            ) -> Result<solarium_client::wire::Instruction> {
                                let instruction_data = solarium_client::Instruction::new(solarium::discriminator!(#discriminator), (#(#arguments),*,));
                                let instruction_data = solarium_client::prelude::borsh::to_vec(&instruction_data)?;
                                Ok(solarium_client::wire::Instruction::new_with_bytes(
                                    program_address,
                                    &instruction_data,
                                    vec![#(#accounts),*],
                                ))
                            }
                        };
                        client_methods.push(client_method);
                        let message_builder_method = quote! {
                            pub fn #method_name(mut self, #(#parameters),*) -> Result<Self> {
                                let instruction = #client::#instruction_with_address(self.1, #(#message_builder_arguments),*)?;
                                self.message.instructions.push(instruction);
                                Ok(self)
                            }
                        };
                        message_builder_methods.push(message_builder_method);
                    }
                }

                items.push(quote!(
                    pub use #program_crate::*;
                    #[allow(unused_imports)]
                    use solarium_client::prelude::*;

                    #client_base

                    impl #client {
                        #[allow(unused_imports)]
                        #(#client_methods)*
                    }

                    impl #message_builder {
                        #(#message_builder_methods)*
                    }
                ));
            }
        }

        Ok(syn::parse_quote! {
            pub mod #ident {
                #(#items)*
            }
        })
    }
}
