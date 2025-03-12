use anchor_lang_idl_spec::{Idl, IdlDefinedFields, IdlField, IdlSerialization, IdlType, IdlTypeDef, IdlTypeDefTy};
use anyhow::{Context, Result};
use ligen_ir::{Identifier, KindDefinition, Type, TypeDefinition};
use solarium_workspace::Workspace;

fn get_idl_type(type_: &Type) -> IdlType {
    if *type_ == Type::u8() {
        IdlType::U8
    } else if *type_ == Type::u16() {
        IdlType::U16
    } else if *type_ == Type::u32() {
        IdlType::U32
    } else if *type_ == Type::u64() {
        IdlType::U64
    } else if *type_ == Type::u128() {
        IdlType::U128
    } else if *type_ == Type::i8() {
        IdlType::I8
    } else if *type_ == Type::i16() {
        IdlType::I16
    } else if *type_ == Type::i32() {
        IdlType::I32
    } else if *type_ == Type::i64() {
        IdlType::I64
    } else if *type_ == Type::i128() {
        IdlType::I128
    } else if *type_ == Type::f32() {
        IdlType::F32
    } else if *type_ == Type::f64() {
        IdlType::F64
    } else if *type_ == Type::string() {
        IdlType::String
    } else if *type_ == Type::boolean() {
        IdlType::Bool
    } else {
        IdlType::Bytes
    }
}

pub fn process(input: TypeDefinition) -> Result<()> {
    todo!("Use ligen-anchor-generator to generate the IDL");

    let current_program_id = std::env::var("CARGO_PKG_NAME").expect("Failed to get current program ID");
    let workspace = Workspace::current()?;

    let idl = Identifier::from(current_program_id).to_snake_case();
    let idl_path = workspace.root.join("target").join("idl").join(format!("{}.json", idl));
    let idl = std::fs::read_to_string(&idl_path)?;
    let mut idl: Idl = serde_json::from_str(&idl)?;

    let name = input.identifier.to_string();

    let docs = input.attributes.get_documentation();

    let serialization = IdlSerialization::Borsh;
    let repr = None;
    let generics = Default::default();

    let mut named_fields = vec![];
    let mut tuple_fields = vec![];

    if let KindDefinition::Structure(structure) = &input.definition {
        for field in &structure.fields {
            if let Some(identifier) = &field.identifier {
                let name = identifier.to_string();
                let docs = field.attributes.get_documentation();
                let ty = get_idl_type(&field.type_);

                named_fields.push(IdlField {
                    docs,
                    name,
                    ty,
                });
            } else {
                let ty = get_idl_type(&field.type_);
                tuple_fields.push(ty);
            }
        }
    }
    
    let fields = if !named_fields.is_empty() {
        Some(IdlDefinedFields::Named(named_fields))
    } else if !tuple_fields.is_empty() {
        Some(IdlDefinedFields::Tuple(tuple_fields))
    } else {
        None
    };

    idl.types.push(IdlTypeDef {
        name,
        docs,
        serialization,
        repr,
        generics,
        ty: IdlTypeDefTy::Struct { fields },
    });

    std::fs::write(&idl_path, serde_json::to_string_pretty(&idl)?).context("Failed to write IDL")?;
    Ok(())
}
