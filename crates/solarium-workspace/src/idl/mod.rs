use ligen::idl::{Identifier, Library};
use ligen::prelude::*;
use ligen_anchor::generator::AnchorGenerator;
use ligen_rust::parser::RustLibraryParser;

use crate::prelude::{Result, *};
use crate::Program;
use crate::Workspace;

#[derive(Debug)]
pub struct Idl {
    pub idl: Library,
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
                std::fs::write(
                    idl_path,
                    serde_json::to_string_pretty(&idl).context("Failed to write IDL")?,
                )
                .context("Failed to write IDL")?;
            }
        }
        Ok(())
    }

    /// The program's interface, and only that: the `#[program]` block in the crate root and the
    /// imports it is written against, read from the program crate in `folder`. What a client is
    /// generated from.
    ///
    /// The rest of the crate is not read. A client needs none of it — its argument types are named
    /// by the paths the block writes, and resolved by the compiler — and reading it all means
    /// evaluating every constant and every array length in every module, at every build of every
    /// crate that generates a client. Unlike building one from a `Program`, this names no address
    /// and so touches no keypair: the program's folder may be a dependency's, not ours to write in.
    pub fn program_interface(folder: impl AsRef<std::path::Path>) -> Result<Self> {
        let folder = folder.as_ref();
        let root = folder.join("src").join("lib.rs");
        let source = std::fs::read_to_string(&root)
            .with_context(|| format!("Failed to read {}", root.display()))?;
        let file = syn::parse_file(&source)
            .map_err(|error| anyhow::anyhow!("Failed to parse {}: {error}", root.display()))?;
        let items = file
            .items
            .into_iter()
            .filter(|item| matches!(item, syn::Item::Impl(_) | syn::Item::Use(_)))
            .collect();
        let name = ligen::idl::Identifier::from(
            folder
                .file_name()
                .and_then(|name| name.to_str())
                .context("A program folder has no name")?,
        );
        let module: syn::ItemMod = syn::parse_quote! { pub mod root {} };
        let module = syn::ItemMod {
            attrs: file.attrs,
            content: Some((Default::default(), items)),
            ..module
        };
        let root_module = ligen_rust::parser::RustModuleParser::new()
            .transform(module, &Config::default())?;
        let idl = Library { identifier: name, metadata: Default::default(), root_module };
        Ok(Idl { idl })
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
        idl.metadata
            .table
            .insert("address".to_string(), program.public_key.to_string());
        Ok(Idl { idl })
    }
}
