use anyhow::Result;
use ligen::generator::Generator;
use ligen::idl::Identifier;
use ligen_rust::generator::{RustIdentifierGenerator, RustTypeGenerator};
use quote::quote;

use super::instruction_attribute::{self, InstructionAttribute};

#[allow(non_snake_case)]
pub fn generate(
    program_impl: &mut syn::ItemImpl,
    input: &ligen::idl::Interface,
) -> Result<proc_macro2::TokenStream> {
    let program_name = program_impl.self_ty.clone();
    let config = Default::default();
    let identifier_generator = RustIdentifierGenerator::default();
    let type_generator = RustTypeGenerator::default();
    let instruction_name = Identifier::from(format!("{}Instruction", input.identifier));
    let instruction_name = identifier_generator.generate(&instruction_name, &config)?;

    let mut constants = Vec::new();
    let mut variants = Vec::new();
    let mut deserializers = Vec::new();
    let mut serializers = Vec::new();
    let mut instructions_parameters = Vec::new();
    let mut calls = Vec::new();
    let mut calls_pinocchio = Vec::new();
    let mut entries = Vec::new();
    let mut entries_pinocchio = Vec::new();

    let mut attributes = std::collections::HashMap::new();
    for item in &mut program_impl.items {
        if let syn::ImplItem::Fn(method) = item {
            let attribute = InstructionAttribute::parse(method)?;
            InstructionAttribute::strip(method);
            // Called from its own entry and nowhere else, so inlining it costs nothing — and it
            // lets whatever the method calls once be inlined in turn, finding its accounts in the
            // slice instead of being handed each one.
            method.attrs.push(syn::parse_quote!(#[inline(always)]));
            // Argument types as written. The interface's own rendering of a type does not survive
            // every shape — an array comes back as a name — and these are what Borsh has to read.
            let types: Vec<syn::Type> = method
                .sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    syn::FnArg::Typed(typed) => Some((*typed.ty).clone()),
                    syn::FnArg::Receiver(_) => None,
                })
                .collect();
            attributes.insert(
                method.sig.ident.to_string(),
                (method.sig.ident.clone(), attribute, types),
            );
        }
    }
    instruction_attribute::check_unique(
        attributes
            .values()
            .flat_map(|(ident, attribute, _)| attribute.all().map(move |d| (ident, d))),
    )?;

    for method in &input.methods {
        let identifier = &method.identifier;
        let MethodName = identifier_generator.generate(&identifier.to_pascal_case(), &config)?;
        let METHOD_NAME =
            identifier_generator.generate(&identifier.to_screaming_snake_case(), &config)?;
        let method_name = identifier_generator.generate(&identifier.to_snake_case(), &config)?;
        let (_, attribute, types) = attributes
            .get(&identifier.to_string())
            .expect("every parsed method is an item of the impl");
        let discriminator = attribute.discriminator;
        let aliases = &attribute.aliases;
        let parameter_structure = identifier_generator.generate(
            &Identifier::from(format!(
                "{}{}",
                input.identifier.to_string(),
                identifier.to_pascal_case()
            )),
            &config,
        )?;
        constants.push(quote! { pub const #METHOD_NAME: u64 = #discriminator; });
        variants.push(quote! { #MethodName(#parameter_structure) });
        deserializers.push(quote! {
            Self::#METHOD_NAME #(| #aliases)* => Ok(Self::#MethodName(#parameter_structure::deserialize_reader(reader)?))
        });
        serializers.push(quote! {
            Self::#MethodName(value) => {
                Self::#METHOD_NAME.serialize(writer)?;
                value.serialize(writer)
            }
        });

        let mut inputs = Vec::new();
        let mut arguments = Vec::new();
        let mut arguments_pinocchio = Vec::new();
        // Accounts are indexed after a single length check rather than drawn one at a time: the
        // same refusal for a short list, without a bounds check and an error path per account.
        let mut named_accounts = 0usize;
        for (position, input) in method.inputs.iter().enumerate() {
            if input.type_.is_constant_reference() || input.type_.is_mutable_reference() {
                let inner_type = input
                    .type_
                    .path
                    .last()
                    .generics
                    .types
                    .first()
                    .expect("Reference must have a target type");
                let type_ = type_generator.generate(inner_type, &config)?;
                if inner_type.path.last().identifier == "Remaining" {
                    // It takes every account left, so nothing named after it could be reached.
                    let accounts_after = method.inputs[position + 1..].iter().any(|input| {
                        input.type_.is_constant_reference() || input.type_.is_mutable_reference()
                    });
                    if accounts_after {
                        anyhow::bail!("`{identifier}`: `Remaining` must be the last account");
                    }
                    arguments.push(quote! { &solarium_program::Remaining::new(&accounts[#named_accounts..]) });
                    arguments_pinocchio.push(quote! { &solarium_program::Remaining::rest(accounts) });
                } else if input.type_.is_mutable_reference() {
                    arguments.push(quote! {
                        &mut <#type_>::try_from(&accounts[#named_accounts])?
                    });
                    arguments_pinocchio.push(quote! {
                        &mut <#type_>::try_from(accounts.next().ok_or(solarium_program::ProgramError::NotEnoughAccountKeys)?)?
                    });
                } else {
                    arguments.push(quote! {
                        &<#type_>::try_from(&accounts[#named_accounts])?
                    });
                    arguments_pinocchio.push(quote! {
                        &<#type_>::try_from(accounts.next().ok_or(solarium_program::ProgramError::NotEnoughAccountKeys)?)?
                    });
                }
                if inner_type.path.last().identifier != "Remaining" {
                    named_accounts += 1;
                }
            } else {
                let input_name = identifier_generator.generate(&input.identifier, &config)?;
                let input_type = &types[position];
                inputs.push(quote! {
                    pub #input_name: #input_type
                });
                arguments.push(quote! {
                    arguments.#input_name
                });
                arguments_pinocchio.push(quote! {
                    arguments.#input_name
                });
            }
        }

        instructions_parameters.push(quote! {
            #[derive(solarium::prelude::borsh::BorshDeserialize, solarium::prelude::borsh::BorshSerialize)]
            #[borsh(crate = "solarium::prelude::borsh")]
            pub struct #parameter_structure {
                #(#inputs),*
            }
        });

        let length_check = (named_accounts > 0).then(|| quote! {
            if accounts.len() < #named_accounts {
                return Err(solarium_program::prelude::solana_program::program_error::ProgramError::NotEnoughAccountKeys.into());
            }
        });
        // Each instruction is entered through a function of its own, handed the account slice
        // whole. Inlined into one dispatcher, every method would share its stack frame — on sBPF
        // a hard 4 KiB — and every call a method makes would be handed its accounts one by one
        // past the few registers there are, rather than find them in the slice where they lie.
        let entry = quote::format_ident!("__solarium_{}", method_name.to_string());
        entries.push(quote! {
            #[inline(never)]
            #[doc(hidden)]
            fn #entry<'a>(
                &self,
                accounts: &'a [solarium_program::prelude::solana_program::account_info::AccountInfo<'a>],
                arguments: #parameter_structure,
            ) -> Result<()> {
                #length_check
                self.#method_name(#(#arguments),*)
            }
        });
        calls.push(quote! {
            #instruction_name::#MethodName(arguments) => self.#entry(accounts, arguments)?
        });
        entries_pinocchio.push(quote! {
            #[inline(never)]
            #[doc(hidden)]
            fn #entry<'a>(
                &self,
                accounts: &'a mut [solarium_program::prelude::pinocchio::AccountView],
                arguments: #parameter_structure,
            ) -> Result<()> {
                let accounts = &mut accounts.iter_mut();
                self.#method_name(#(#arguments_pinocchio),*)
            }
        });
        calls_pinocchio.push(quote! {
            #instruction_name::#MethodName(arguments) => self.#entry(accounts, arguments)?
        });
    }

    let instruction_parameters = quote! {
        #(#instructions_parameters)*
    };

    let instruction_enum = quote! {
        #[repr(u64)]
        pub enum #instruction_name {
            #(#variants),*
        }
    };

    let constants = quote! {
        impl #instruction_name {
            #(#constants)*
        }
    };

    let deserialize = quote! {
        impl solarium::prelude::borsh::BorshDeserialize for #instruction_name {
            fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let discriminant = u64::deserialize_reader(reader)?;
                match discriminant {
                    #(#deserializers),*,
                    _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid instruction")),
                }
            }
        }
    };

    let serialize = quote! {
        impl solarium::prelude::borsh::BorshSerialize for #instruction_name {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                match self {
                    #(#serializers),*
                }
            }
        }
    };

    let process_instruction = quote! {
        impl #program_name {
            pub fn process_instruction<'a>(
                &self,
                program_id: &solarium_program::prelude::solana_program::pubkey::Pubkey,
                accounts: &'a [solarium_program::prelude::solana_program::account_info::AccountInfo<'a>],
                instruction_data: &[u8],
            ) -> Result<()> {
                check_id(program_id).then_some(()).ok_or(solarium_program::prelude::solana_program::program_error::ProgramError::IncorrectProgramId)?;
                match Self::instruction(instruction_data).map_err(|_| solarium_program::prelude::solana_program::program_error::ProgramError::InvalidInstructionData)? {
                    #(#calls),*
                }
                Ok(())
            }

            #(#entries)*
        }
    };

    let process_instruction_pinocchio = quote! {
        impl #program_name {
            pub fn process_instruction(
                &self,
                program_id: &solarium_program::prelude::pinocchio::Address,
                accounts: &mut [solarium_program::prelude::pinocchio::AccountView],
                instruction_data: &[u8],
            ) -> Result<()> {
                let program_id = solarium_program::Pubkey::new_from_array(program_id.to_bytes());
                check_id(&program_id)
                    .then_some(())
                    .ok_or(solarium_program::ProgramError::IncorrectProgramId)?;
                match Self::instruction(instruction_data)
                    .map_err(|_| solarium_program::ProgramError::InvalidInstructionData)?
                {
                    #(#calls_pinocchio),*
                }
                Ok(())
            }

            #(#entries_pinocchio)*
        }
    };

    let program_definition = quote! {
        pub struct #program_name;

        impl #program_name {
            /// Reads the instruction an input names.
            ///
            /// Whatever follows its arguments is left unread rather than refused, as Anchor does:
            /// a program calling back into this one — a settle, an oracle — may append bytes of
            /// its own that the method has no use for.
            pub fn instruction(instruction_data: &[u8]) -> std::io::Result<#instruction_name> {
                <#instruction_name as solarium::prelude::borsh::BorshDeserialize>::deserialize(
                    &mut &instruction_data[..],
                )
            }
        }
    };

    let output = quote! {
        #instruction_parameters
        #instruction_enum
        #constants
        #deserialize
        #serialize
        #program_definition

        #[cfg(all(not(target_arch = "wasm32"), not(feature = "pinocchio")))]
        #process_instruction

        #[cfg(all(not(target_arch = "wasm32"), feature = "pinocchio"))]
        #process_instruction_pinocchio

        #[cfg(not(target_arch = "wasm32"))]
        #program_impl
    };
    Ok(output)
}
