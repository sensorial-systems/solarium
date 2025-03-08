use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::prelude::*;
use ligen_ir::Identifier;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};

use crate::idl::Idl;
use crate::Workspace;

pub struct Program {
    pub name: Identifier,
    pub public_key: Pubkey,
    /// The folder where you will find the Cargo.toml file for the program
    pub root: PathBuf,
}

impl Program {
    fn toml(&self) -> Result<toml::Value> {
        let content = std::fs::read_to_string(self.root.join("Cargo.toml")).context("Failed to read Cargo.toml")?;
        toml::from_str(&content).context("Failed to parse Cargo.toml")
    }

    fn workspace_toml(&self) -> Result<toml::Value> {
        let path = &self.root;
        let mut path_ancestors = path.as_path().ancestors();
    
        while let Some(p) = path_ancestors.next() {
            let has_cargo =
                std::fs::read_dir(p)?
                    .into_iter()
                    .any(|p| p.unwrap().file_name() == OsString::from("Cargo.lock"));
            if has_cargo {
                let workspace_root = PathBuf::from(p);
                let toml = std::fs::read_to_string(workspace_root.join("Cargo.toml"))
                    .context("Failed to read Cargo.toml")?;
                return toml::from_str(&toml).context("Failed to parse Cargo.toml");
            }
        }
        Err(anyhow::anyhow!("Failed to find workspace root"))
    }

    pub fn workspace_package(&self, table: &str, key: &str) -> Option<toml::Value> {
        let toml = self.workspace_toml().ok()?;
        toml
            .get("workspace")
            .and_then(|v| v.get(table))
            .and_then(|v| v.get(key))
            .cloned()
    }

    pub fn package(&self, table: &str, key: &str) -> Option<toml::Value> {
        let toml = self.toml().ok()?;
        let value = toml.get(table).and_then(|v| v.get(key));
        if let Some(value) = value {
            match value {
                toml::Value::Table(toml_table) => {
                    if let Some(workspace) = toml_table.get("workspace") {
                        if workspace.as_bool().unwrap_or(false) {
                            return self.workspace_package(table, key);
                        }
                    }
                },
                _ => return Some(value.clone()),
            }
        }
        None
    }


    pub fn contact(&self) -> Option<String> {
        self.package("package", "authors")
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|v|
                v
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s|
                        s
                            .to_string()
                            .replace("\"", "")
                        )
                    .collect::<Vec<_>>()
                    .join(", ")
            )
    }

    pub fn description(&self) -> Option<String> {
        self.package("package", "description")
            .as_ref()
            .and_then(|v| v.as_str())
            .map(|s| s.to_string().replace("\"", ""))
    }

    pub fn repository(&self) -> Option<String> {
        self.package("package", "repository")
            .as_ref()
            .and_then(|v| v.as_str())
            .map(|s| s.to_string().replace("\"", ""))
    }

    pub fn version(&self) -> Option<String> {
        self.package("package", "version")
            .as_ref()
            .and_then(|v| v.as_str())
            .map(|s| s.to_string().replace("\"", ""))
    }

    pub async fn idl(&self) -> Result<Idl> {
        Idl::try_from(self)
    }

    pub async fn deploy(&self, workspace: &Workspace) -> Result<()> {
        let program_so = workspace.root.join("target").join("deploy").join(format!("{}.so", self.name.to_snake_case()));
        let status = tokio::process::Command::new("solana")
            .arg("program")
            .arg("deploy")
            .arg(program_so)
            .status()
            .await
            .context("failed to run solana program deploy")?;

        if !status.success() {
            anyhow::bail!("solana program deploy failed");
        }

        Ok(())
    }

    pub fn look_for_cargo_toml(root: &Path, name: impl AsRef<str>) -> Result<PathBuf> {
        let name = name.as_ref();
        walkdir::WalkDir::new(root)
            .into_iter()
            .find_map(|entry| {
                if let Ok(entry) = entry {
                    if entry.path().extension().map_or(false, |ext| ext == "toml") {
                        return
                            std::fs::read_to_string(entry.path())
                                .ok()
                                .and_then(|content| toml::from_str::<toml::Value>(&content).ok())
                                .and_then(|value|
                                    value.get("package")
                                        .and_then(|v| v.get("name"))
                                        .and_then(|v| v.as_str())
                                        .filter(|package_name| *package_name == name)
                                        .map(|_| entry.path().to_path_buf())
                                );
                    }
                }
                None
            })
            .context("Failed to find cargo.toml")
    }

    pub fn try_from(root: &Path, path: PathBuf) -> Result<Self> {
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") && path.file_name().unwrap().to_string_lossy().to_string().contains("-keypair") {
            let keypair = Keypair::read_from_file(&path).map_err(|e| anyhow::anyhow!("Failed to load keypair: {}", e))?;
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let name = name.replace("-keypair.json", "");
            let name = Identifier::from(name);
            let name = name.to_kebab_case();
            let root = Self::look_for_cargo_toml(root, name.to_string())?
                .parent()
                .context("Failed to find program root")?
                .to_path_buf();
            let public_key = keypair.pubkey();
            Ok(Program { name, public_key, root })
        } else {
            Err(anyhow::anyhow!("Invalid program path: {}", path.display()))
        }
    }
}
