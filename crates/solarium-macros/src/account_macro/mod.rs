use anyhow::{Context, Result};
use ligen_anchor_generator::AnchorTypeDefinitionGenerator;
use ligen_generator::Generator;
use ligen_ir::TypeDefinition;
use solarium_workspace::Workspace;

pub fn process(input: TypeDefinition) -> Result<()> {
    let workspace = Workspace::current().context("Failed to get workspace")?;
    let program = std::env::var("CARGO_PKG_NAME").expect("Failed to get current program ID");
    let program = workspace.program(&program).context("Failed to get program")?;

    let mut idl = program.anchor_idl_from_file(&workspace).context("Failed to get program IDL")?;
    if idl.types.iter().any(|t| t.name == input.identifier.to_string()) {
        return Ok(());
    }
    let idl_type = AnchorTypeDefinitionGenerator::new().generate(&input, &Default::default())?;
    idl.types.push(idl_type);

    std::fs::write(&program.idl_path(&workspace), serde_json::to_string_pretty(&idl)?).context("Failed to write IDL")?;
    Ok(())
}
