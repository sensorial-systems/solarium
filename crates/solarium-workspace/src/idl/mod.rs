use ligen_anchor_generator::AnchorGenerator;
use ligen::prelude::*;
use ligen::ir::Identifier;
use ligen_rust_parser::library::RustLibraryParser;

use crate::prelude::{*, Result};
use crate::Program;
use crate::Workspace;

#[derive(Debug)]
pub struct Idl {
    pub idl: ligen::ir::Library
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdlType {
    Ligen,
    Anchor,
}

impl Idl {
    pub fn save_as(&self, workspace: &Workspace, idl_type: IdlType) -> Result<()> {
        match idl_type {
            IdlType::Ligen => {
                todo!("Save as Ligen IDL");
            }
            IdlType::Anchor => {
                let generator = AnchorGenerator::new();
                let idl = generator.generate(&self.idl, &Config::default())?;
                let idl_path = workspace.root.join("target").join("idl");
                std::fs::create_dir_all(&idl_path).context("Failed to create IDL directory")?;
                let name = Identifier::from(idl.metadata.name.clone()).to_snake_case();
                let idl_path = idl_path.join(format!("{}.json", name));
                std::fs::write(idl_path, serde_json::to_string_pretty(&idl).context("Failed to write IDL")?).context("Failed to write IDL")?;
            }
        }
        Ok(())
    }

    pub fn deploy(&self, _workspace: &Workspace) -> Result<()> {
        Ok(())
    }
}

impl TryFrom<&Program> for Idl {
    type Error = anyhow::Error;

    fn try_from(program: &Program) -> Result<Self> {
        let parser = RustLibraryParser::new();
        let mut idl = parser.transform(&program.folder, &Config::default())?;
        idl.metadata.table.insert("address".to_string(), program.public_key.to_string());
        Ok(Idl { idl })
    }
}