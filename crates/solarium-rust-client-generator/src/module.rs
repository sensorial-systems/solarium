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
                        let discriminator = discriminator(method)?;
                        let mut parameters: Vec<proc_macro2::TokenStream> = Vec::new();
                        let mut arguments: Vec<proc_macro2::TokenStream> = Vec::new();
                        let mut message_builder_arguments: Vec<proc_macro2::TokenStream> =
                            Vec::new();
                        let mut accounts = Vec::new();
                        let mut remaining = None;
                        for parameter in &method.inputs {
                            let name = &parameter.identifier;
                            let name = self.identifier_generator.generate(&name, config)?;
                            message_builder_arguments.push(quote! {
                                #name
                            });
                            let is_remaining = parameter
                                .type_
                                .path
                                .last()
                                .generics
                                .types
                                .first()
                                .is_some_and(|inner| inner.path.last().identifier == "Remaining");
                            if is_remaining {
                                parameters.push(quote! {
                                    #name: impl IntoIterator<Item = solarium_client::wire::AccountMeta>
                                });
                                remaining = Some(name);
                            } else if parameter.type_.is_mutable_reference()
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
                                let (parameter, argument) =
                                    value_parameter(&name, &parameter.type_, &type_);
                                parameters.push(parameter);
                                arguments.push(argument);
                            }
                        }
                        if arguments.is_empty() {
                            arguments.push(quote! { () });
                        }
                        let remaining = remaining.map(|name| quote! { accounts.extend(#name); });
                        let client_method = quote! {
                            pub fn #instruction(#(#parameters),*) -> Result<solarium_client::wire::Instruction> {
                                Self::#instruction_with_address(#program_id, #(#message_builder_arguments),*)
                            }

                            fn #instruction_with_address(
                                program_address: solarium_client::wire::Pubkey,
                                #(#parameters),*
                            ) -> Result<solarium_client::wire::Instruction> {
                                let instruction_data = solarium_client::Instruction::new(#discriminator, (#(#arguments),*,));
                                let instruction_data = solarium_client::prelude::borsh::to_vec(&instruction_data)?;
                                #[allow(unused_mut)]
                                let mut accounts = vec![#(#accounts),*];
                                #remaining
                                Ok(solarium_client::wire::Instruction::new_with_bytes(
                                    program_address,
                                    &instruction_data,
                                    accounts,
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

/// How a client takes one of an instruction's values, and how it passes it on.
///
/// `impl Into<T>` wherever a conversion is worth having — `&str` for a `String`, an array for a
/// `Pubkey` — but never for a bare number. An integer literal there has nothing to infer itself
/// from: it falls back to `i32`, which converts into no other integer type, so `times(10)` against
/// a `u32` is a compile error and the caller has to write `10u32`. A scalar is taken as itself.
///
/// The argument follows from that. Behind `impl Into<T>` there is one candidate and `.into()`
/// resolves to it; a scalar is already the type wanted, and `.into()` on it would be ambiguous
/// rather than free — the arguments go into a tuple that `Instruction::new` is generic over, so
/// nothing downstream would pin the target down.
fn value_parameter(
    name: &syn::Ident,
    type_: &ligen::idl::Type,
    rendered: &syn::Type,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if is_scalar(type_) {
        (quote! { #name: #rendered }, quote! { #name })
    } else {
        (
            quote! { #name: impl Into<#rendered> },
            quote! { #name.into() },
        )
    }
}

/// Whether `type_` is a bare number-like primitive, as opposed to something a value converts into.
///
/// Numbers only, and only bare ones. `Vec<u64>` and `[u8; 32]` keep `impl Into<T>`, and so do
/// `bool` and `char`: a literal of any of those is typed by its own contents, so nothing is left
/// for inference to guess at. It is the numbers that have a fallback to go wrong.
fn is_scalar(type_: &ligen::idl::Type) -> bool {
    const SCALARS: [&str; 14] = [
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64",
    ];
    let last = type_.path.last();
    last.generics.types.is_empty() && SCALARS.iter().any(|scalar| last.identifier == *scalar)
}

/// The bytes a client starts `method`'s instruction with — the same ones `#[program]` dispatches
/// on: `#[instruction(discriminator = ...)]` when the method gives one, `"global:<name>"` hashed
/// when it does not.
fn discriminator(method: &ligen::idl::Method) -> Result<proc_macro2::TokenStream> {
    let given = method
        .attributes
        .get_group("instruction")
        .and_then(|group| group.get_named("discriminator"));
    Ok(match given {
        None => {
            let namespace = format!("global:{}", method.identifier);
            quote! { solarium::discriminator!(#namespace) }
        }
        Some(ligen::idl::Literal::String(namespace)) => {
            quote! { solarium::discriminator!(#namespace) }
        }
        Some(ligen::idl::Literal::UnsignedInteger(value)) => {
            quote! { (#value as u64).to_le_bytes() }
        }
        Some(ligen::idl::Literal::Integer(value)) if *value >= 0 => {
            let value = *value as u64;
            quote! { (#value as u64).to_le_bytes() }
        }
        Some(other) => {
            return Err(ligen::common::Error::Message(format!(
                "`{}`: a discriminator is an integer or a string to hash, not {other:?}",
                method.identifier
            )))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ligen::transformer::Transformer;
    use ligen_rust::parser::RustInterfaceParser;

    fn methods() -> Vec<ligen::idl::Method> {
        let program: syn::ItemImpl = syn::parse_quote! {
            impl Game {
                pub fn plain(&self) -> Result<()> { Ok(()) }
                #[instruction(discriminator = 16)]
                pub fn numbered(&self) -> Result<()> { Ok(()) }
                #[instruction(discriminator = "global:process_undelegation", alias = 3)]
                pub fn named(&self) -> Result<()> { Ok(()) }
            }
        };
        RustInterfaceParser::new()
            .transform(program, &Default::default())
            .unwrap()
            .methods
    }

    fn inputs() -> Vec<ligen::idl::Parameter> {
        let program: syn::ItemImpl = syn::parse_quote! {
            impl Game {
                pub fn play(
                    &self,
                    prompt: String,
                    times: u32,
                    stake: u64,
                    odds: f32,
                    loud: bool,
                    seeds: Vec<Vec<u8>>,
                    randomness: [u8; 32],
                ) -> Result<()> {
                    Ok(())
                }
            }
        };
        RustInterfaceParser::new()
            .transform(program, &Default::default())
            .unwrap()
            .methods
            .remove(0)
            .inputs
    }

    #[test]
    fn a_number_is_taken_as_itself_and_anything_else_by_conversion() {
        let rendered: Vec<(String, bool)> = inputs()
            .iter()
            .map(|input| (input.identifier.to_string(), is_scalar(&input.type_)))
            .collect();
        assert_eq!(
            rendered,
            vec![
                ("prompt".to_string(), false),
                ("times".to_string(), true),
                ("stake".to_string(), true),
                ("odds".to_string(), true),
                // A literal of any of these types itself, so a conversion costs the caller
                // nothing and an integer's fallback never comes into it.
                ("loud".to_string(), false),
                ("seeds".to_string(), false),
                ("randomness".to_string(), false),
            ]
        );
    }

    #[test]
    fn a_number_is_passed_on_without_a_conversion_to_infer() {
        let name: syn::Ident = syn::parse_quote!(times);
        let rendered: syn::Type = syn::parse_quote!(u32);
        let scalar = ligen::idl::Type::from(ligen::idl::Path::from("u32"));
        let (parameter, argument) = value_parameter(&name, &scalar, &rendered);
        assert_eq!(parameter.to_string(), quote! { times: u32 }.to_string());
        assert_eq!(argument.to_string(), quote! { times }.to_string());

        let name: syn::Ident = syn::parse_quote!(prompt);
        let rendered: syn::Type = syn::parse_quote!(String);
        let string = ligen::idl::Type::from(ligen::idl::Path::from("String"));
        let (parameter, argument) = value_parameter(&name, &string, &rendered);
        assert_eq!(
            parameter.to_string(),
            quote! { prompt: impl Into<String> }.to_string()
        );
        assert_eq!(argument.to_string(), quote! { prompt.into() }.to_string());
    }

    #[test]
    fn a_client_sends_what_the_program_dispatches_on() {
        let generated: Vec<String> = methods()
            .iter()
            .map(|method| discriminator(method).unwrap().to_string())
            .collect();
        assert_eq!(
            generated[0],
            quote! { solarium::discriminator!("global:plain") }.to_string()
        );
        assert_eq!(
            generated[1],
            quote! { (16u64 as u64).to_le_bytes() }.to_string()
        );
        assert_eq!(
            generated[2],
            quote! { solarium::discriminator!("global:process_undelegation") }.to_string()
        );
    }
}
