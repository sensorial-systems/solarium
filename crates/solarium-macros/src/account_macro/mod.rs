use anyhow::{Context, Result};
use ligen_anchor_generator::AnchorTypeDefinitionGenerator;
use ligen_generator::Generator;
use ligen_ir::{Identifier, TypeDefinition};
use solarium_workspace::Workspace;

pub fn process(input: TypeDefinition) -> Result<()> {
    let current_program_name = std::env::var("CARGO_PKG_NAME").expect("Failed to get current program ID");
    let workspace = Workspace::current().context("Failed to get workspace")?;
    let program = workspace.program(&current_program_name).context("Failed to get program")?;

    let idl = Identifier::from(current_program_name).to_snake_case();
    let idl_path = workspace.root.join("target").join("idl").join(format!("{}.json", idl));
    if !idl_path.exists() {
        let idl = program.idl().context("Failed to get program IDL")?;
        idl.save_as(&workspace, solarium_workspace::IdlType::Anchor).context("Failed to save IDL")?;
    }
    let idl = std::fs::read_to_string(&idl_path).context("Failed to read IDL")?;
    let mut idl: anchor_lang_idl_spec::Idl = serde_json::from_str(&idl).context("Failed to parse IDL")?;

    if idl.types.iter().any(|t| t.name == input.identifier.to_string()) {
        return Ok(());
    }

    let idl_type = AnchorTypeDefinitionGenerator::new().generate(&input, &Default::default())?;

    idl.types.push(idl_type);

    std::fs::write(&idl_path, serde_json::to_string_pretty(&idl)?).context("Failed to write IDL")?;
    Ok(())
}
