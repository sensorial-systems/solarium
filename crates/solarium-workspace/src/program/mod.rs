use std::path::{Path, PathBuf};

use crate::prelude::*;
use ligen_ir::Identifier;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};

use crate::idl::Idl;
use crate::Workspace;

#[derive(Debug, Clone)]
pub struct Program {
    pub name: Identifier,
    pub public_key: Pubkey,
    /// The folder where you will find the Cargo.toml file for the program
    pub root: PathBuf,
}

impl Program {
    pub fn get_keypair_path_for_name(workspace: &Workspace, name: impl AsRef<str>) -> Result<PathBuf> {
        let name = name.as_ref();
        let deploy = workspace.root.join("target").join("deploy");
        let name = Identifier::new(name).to_snake_case();
        let keypair_file = deploy.join(format!("{}-keypair.json", name));
        Ok(keypair_file)
    }

    pub fn create_keypair(workspace: &Workspace, name: impl AsRef<str>) -> Result<Keypair> {
        let name = name.as_ref();
        let keypair_file = Self::get_keypair_path_for_name(workspace, name)?;
        let keypair = Keypair::new();
        keypair.write_to_file(&keypair_file).map_err(|e| anyhow::anyhow!("Failed to write keypair: {}", e))?;
        Ok(keypair)
    }

    /// Get the program ID from a keypair file. If the keypair file does not exist, create a new one and return the public key.
    pub fn get_program_id_from_file(workspace: &Workspace, name: impl AsRef<str>) -> Result<Pubkey> {
        let keypair_file = Self::get_keypair_path_for_name(workspace, name)?;
        if keypair_file.exists() {
            let keypair = Keypair::read_from_file(keypair_file).map_err(|e| anyhow::anyhow!("Failed to read keypair: {}", e))?;
            Ok(keypair.pubkey())
        } else {
            let keypair = Keypair::new();
            keypair.write_to_file(&keypair_file).map_err(|e| anyhow::anyhow!("Failed to write keypair: {}", e))?;
            Ok(keypair.pubkey())
        }
    }

    pub fn idl(&self) -> Result<Idl> {
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
