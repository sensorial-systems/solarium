use anyhow::Result;
use ligen_generator::Generator;
use ligen_ir::Identifier;
use ligen_rust_generator::{RustIdentifierGenerator, RustTypeGenerator};
use quote::quote;

pub fn generate(program_impl: &mut syn::ItemImpl, input: &ligen_ir::Interface) -> Result<proc_macro2::TokenStream> {
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
        let method_name = &method.identifier;
        let snake_case_name = identifier_generator.generate(&method_name.to_snake_case(), &config)?;
        let pascal_case_name = identifier_generator.generate(&method_name.to_pascal_case(), &config)?;
        let screaming_case_name = identifier_generator.generate(&method_name.to_screaming_snake_case(), &config)?;
        let namespace = format!("global:{}", method_name);
        let parameter_structure = identifier_generator.generate(&Identifier::from(format!("{}{}", input.identifier.to_string(), method_name.to_pascal_case())), &config)?;
        constants.push(quote! { pub const #screaming_case_name: u64 = u64::from_le_bytes(solarium::discriminator!(#namespace)); });
        variants.push(quote! { #pascal_case_name(#parameter_structure) });
        deserializers.push(quote! {
            Self::#screaming_case_name => Ok(Self::#pascal_case_name(#parameter_structure::deserialize_reader(reader)?))
        });
        serializers.push(quote! {
            Self::#pascal_case_name(value) => {
                Self::#screaming_case_name.serialize(writer)?;
                value.serialize(writer)
            }
        });

        let mut inputs = Vec::new();
        let mut arguments = Vec::new();
        let mut accounts: usize = 0;
        for input in &method.inputs {
            if input.type_.is_constant_reference() || input.type_.is_mutable_reference() {
                let type_ = type_generator.generate(&input.type_, &config)?;
                arguments.push(quote! {
                    #type_::try_from(&accounts[#accounts])?
                });
                accounts += 1;
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
            #instruction_name::#pascal_case_name(arguments) => {
                self.#snake_case_name(#(#arguments),*)?;
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
        impl solarium::prelude::borsh::BorshSerialize for ImageGeneratorInstruction {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                match self {
                    #(#serializers),*
                }
            }
        }
    };

    program_impl.items.push(syn::parse_quote!(
        pub fn process_instruction<'a>(
            &self,
            program_id: &solarium::prelude::solana_program::pubkey::Pubkey,
            accounts: &'a [solarium::prelude::solana_program::account_info::AccountInfo<'a>],
            instruction_data: &[u8],
        ) -> Result<()> {
            check_id(program_id).then_some(()).ok_or(solarium::prelude::solana_program::program_error::ProgramError::IncorrectProgramId)?;
            match <#instruction_name as solarium::prelude::borsh::BorshDeserialize>::try_from_slice(instruction_data).map_err(|_| ProgramError::InvalidInstructionData)? {
                #(#calls),*
            }
            Ok(())
        }
    ));

    let output = quote! {
        #instruction_parameters
        #instruction_enum
        #constants
        #deserialize
        #serialize
        #program_impl
    };
    Ok(output)
}
