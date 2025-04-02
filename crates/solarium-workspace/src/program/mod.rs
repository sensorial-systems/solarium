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
    pub folder: PathBuf,
}

impl Program {
    /// Get the program ID from a keypair file. If the keypair file does not exist, create a new one and return the public key.
    pub fn get_program_id_from_file(workspace: impl AsRef<Path>, name: impl AsRef<str>) -> Result<Pubkey> {
        let name = name.as_ref();
        let deploy = workspace.as_ref().join("target").join("deploy");
        let name = Identifier::new(name).to_snake_case();
        let keypair_file = deploy.join(format!("{}-keypair.json", name));
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

    pub fn try_from(root: &Path, folder: PathBuf) -> Result<Self> {
        let cargo_toml = folder.join("Cargo.toml");
        let cargo_toml = std::fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;
        let cargo_toml: toml::Value = toml::from_str(&cargo_toml).context("Failed to parse Cargo.toml")?;
        let package = cargo_toml.get("package").context("Failed to get package")?;
        let name = package.get("name").context("Failed to get package name")?;
        let name = Identifier::from(name.as_str().context("Failed to get package name")?);
        cargo_toml
            .get("dependencies")
            .context("Failed to get dependencies")?
            .get("solarium")
            .context("Solarium is not a dependency")?;
        let public_key = Self::get_program_id_from_file(&root, name.to_string())?;
        Ok(Self { name, public_key, folder })
    }
}
