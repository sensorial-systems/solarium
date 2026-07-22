use anyhow::Result;
use ligen::generator::Generator;
use ligen::idl::Identifier;
use ligen_rust::generator::{RustIdentifierGenerator, RustTypeGenerator};
use quote::quote;

#[allow(non_snake_case)]
pub fn generate(program_impl: &mut syn::ItemImpl, input: &ligen::idl::Interface) -> Result<proc_macro2::TokenStream> {
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
    for method in &input.methods {
        let identifier = &method.identifier;
        let MethodName = identifier_generator.generate(&identifier.to_pascal_case(), &config)?;
        let METHOD_NAME = identifier_generator.generate(&identifier.to_screaming_snake_case(), &config)?;
        let method_name = identifier_generator.generate(&identifier.to_snake_case(), &config)?;
        let namespace = format!("global:{}", identifier);
        let parameter_structure = identifier_generator.generate(&Identifier::from(format!("{}{}", input.identifier.to_string(), identifier.to_pascal_case())), &config)?;
        constants.push(quote! { pub const #METHOD_NAME: u64 = u64::from_le_bytes(solarium::discriminator!(#namespace)); });
        variants.push(quote! { #MethodName(#parameter_structure) });
        deserializers.push(quote! {
            Self::#METHOD_NAME => Ok(Self::#MethodName(#parameter_structure::deserialize_reader(reader)?))
        });
        serializers.push(quote! {
            Self::#MethodName(value) => {
                Self::#METHOD_NAME.serialize(writer)?;
                value.serialize(writer)
            }
        });

        let mut inputs = Vec::new();
        let mut arguments = Vec::new();
        for input in &method.inputs {
            if input.type_.is_constant_reference() || input.type_.is_mutable_reference() {
                let inner_type = input.type_.path.last().generics.types.first().expect("Reference must have a target type");
                let type_ = type_generator.generate(inner_type, &config)?;
                if input.type_.is_mutable_reference() {
                    arguments.push(quote! {
                        &mut <#type_>::try_from(solarium_program::prelude::solana_program::account_info::next_account_info(accounts)?)?
                    });
                } else {
                    arguments.push(quote! {
                        &<#type_>::try_from(solarium_program::prelude::solana_program::account_info::next_account_info(accounts)?)?
                    });
                }
            } else {
                let input_name = identifier_generator.generate(&input.identifier, &config)?;
                let input_type = type_generator.generate(&input.type_, &config)?;
                inputs.push(quote! {
                    #input_name: #input_type
                });
                arguments.push(quote! {
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

        calls.push(quote! {
            #instruction_name::#MethodName(arguments) => {
                self.#method_name(#(#arguments),*)?;
            }
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
                let accounts = &mut accounts.iter();
                match <#instruction_name as solarium::prelude::borsh::BorshDeserialize>::try_from_slice(instruction_data).map_err(|_| solarium_program::prelude::solana_program::program_error::ProgramError::InvalidInstructionData)? {
                    #(#calls),*
                }
                Ok(())
            }
        }
    };

    let program_definition = quote! {
        pub struct #program_name;
    };

    let output = quote! {
        #instruction_parameters
        #instruction_enum
        #constants
        #deserialize
        #serialize
        #program_definition
        #process_instruction

        #program_impl
    };
    Ok(output)
}
