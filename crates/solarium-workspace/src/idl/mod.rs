use ligen_ir::Identifier;

use crate::prelude::*;
use crate::Program;
use crate::Workspace;

#[derive(Debug, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Idl(pub anchor_lang_idl_spec::Idl);

impl Idl {
    pub fn save(&self, workspace: &Workspace) -> Result<()> {
        let idl_path = workspace.root.join("target").join("idl");
        std::fs::create_dir_all(&idl_path).context("Failed to create IDL directory")?;
        let name = Identifier::from(self.0.metadata.name.clone()).to_snake_case();
        let idl_path = idl_path.join(format!("{}.json", name));
        std::fs::write(idl_path, serde_json::to_string_pretty(&self.0).context("Failed to write IDL")?).context("Failed to write IDL")?;
        Ok(())
    }

    pub fn deploy(&self, _workspace: &Workspace) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<&Program> for Idl {
    type Error = anyhow::Error;

    fn try_from(program: &Program) -> Result<Self, Self::Error> {
        let anchor_idl = anchor_lang_idl_spec::Idl {
            address: program.public_key.to_string(),
            metadata: anchor_lang_idl_spec::IdlMetadata {
                contact: program.contact(),
                description: program.description(),
                name: program.name.to_string(),
                repository: program.repository(),
                spec: anchor_lang_idl_spec::IDL_SPEC.to_string(),
                version: program.version().context("Version not present in Cargo.toml")?,
                dependencies: vec![],
                deployments: None,
            },
            docs: vec![],
            instructions: vec![],
            accounts: vec![],
            events: vec![],
            errors: vec![],
            types: vec![],
            constants: vec![],
        };
        Ok(Idl(anchor_idl))
    }
}