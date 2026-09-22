use std::path::{Path, PathBuf};

use crate::{prelude::*, IdlType};
use ligen::idl::Identifier;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::{EncodableKey, Signer};

use crate::idl::Idl;
use crate::Workspace;

#[derive(Debug, Clone)]
pub struct Program {
    pub name: Identifier,
    pub public_key: Pubkey,
    pub folder: PathBuf,
}

impl Program {
    pub async fn build(&self, workspace: &Workspace) -> Result<()> {
        let output_directory = workspace.root.join("target").join("deploy");
        std::fs::create_dir_all(&output_directory)
            .context("Failed to create the program output directory")?;
        println!("Building {}", self.name);
        let mut command = tokio::process::Command::new("cargo");
        command
            .arg("build-sbf")
            .arg("--manifest-path")
            .arg(self.folder.join("Cargo.toml"))
            .arg("--sbf-out-dir")
            .arg(&output_directory)
            .current_dir(&workspace.root);
        // cargo build-sbf exports CC and AR for the SBF target, and cc-rs picks them up for the
        // crates cargo builds for the host as well, such as the dependencies of a proc macro.
        // cc-rs prefers HOST_* when it is not cross-compiling, so pointing those at the host
        // tools keeps the SBF target on the platform-tools toolchain and everything else on the
        // host one.
        for (variable, tool) in [("HOST_CC", "/usr/bin/cc"), ("HOST_AR", "/usr/bin/ar")] {
            if std::env::var_os(variable).is_none() && Path::new(tool).exists() {
                command.env(variable, tool);
            }
        }
        let status = command
            .status()
            .await
            .with_context(|| format!("Failed to build {}", self.name))?;

        if !status.success() {
            anyhow::bail!("Failed to build {}", self.name);
        }
        Ok(())
    }

    /// Get the program ID from a keypair file. If the keypair file does not exist, create a new one and return the public key.
    pub fn get_program_id_from_file(
        workspace: impl AsRef<Path>,
        name: impl AsRef<str>,
    ) -> Result<Pubkey> {
        let name = name.as_ref();
        let deploy = workspace.as_ref().join("target").join("deploy");
        let name = Identifier::new(name).to_snake_case();
        let keypair_file = deploy.join(format!("{}-keypair.json", name));
        if keypair_file.exists() {
            let keypair = Keypair::read_from_file(keypair_file)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair: {}", e))?;
            Ok(keypair.pubkey())
        } else {
            let keypair = Keypair::new();
            keypair
                .write_to_file(&keypair_file)
                .map_err(|e| anyhow::anyhow!("Failed to write keypair: {}", e))?;
            Ok(keypair.pubkey())
        }
    }

    pub fn idl(&self) -> Result<Idl> {
        Idl::try_from(self)
    }

    pub fn idl_path(&self, workspace: &Workspace) -> PathBuf {
        workspace
            .root
            .join("target")
            .join("idl")
            .join(format!("{}.json", self.name.to_snake_case()))
    }

    pub fn anchor_idl_from_file(&self, workspace: &Workspace) -> Result<anchor_lang_idl_spec::Idl> {
        let idl_path = self.idl_path(workspace);
        if !idl_path.exists() {
            let idl = self.idl().context("Failed to get program IDL")?;
            idl.save_as(&workspace, IdlType::Anchor)
                .context("Failed to save IDL")?;
        }
        let idl = std::fs::read_to_string(&idl_path).context("Failed to read IDL")?;
        let idl: anchor_lang_idl_spec::Idl =
            serde_json::from_str(&idl).context("Failed to parse IDL")?;
        Ok(idl)
    }

    /// Deploys to the cluster the Solana CLI is configured for.
    pub async fn deploy(&self, workspace: &Workspace) -> Result<()> {
        self.deploy_to(workspace, None).await
    }

    /// Deploys to `url`, or to the CLI's configured cluster when there is none. The local
    /// validator is always named: left to the configuration, a deploy meant for it lands on
    /// whatever cluster the CLI happens to point at, paid for by its default keypair.
    pub async fn deploy_to(&self, workspace: &Workspace, url: Option<&str>) -> Result<()> {
        println!("Deploying {} ({})", self.name, self.public_key);
        let program_so = workspace
            .root
            .join("target")
            .join("deploy")
            .join(format!("{}.so", self.name.to_snake_case()));
        let mut command = tokio::process::Command::new("solana");
        if let Some(url) = url {
            command.arg("--url").arg(url);
        }
        let status = command
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
        let cargo_toml =
            std::fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;
        let cargo_toml: toml::Value =
            toml::from_str(&cargo_toml).context("Failed to parse Cargo.toml")?;
        // Only treat crates that build a cdylib as deployable on-chain programs
        let is_program_crate = cargo_toml
            .get("lib")
            .and_then(|lib| lib.get("crate-type"))
            .and_then(|crate_type| crate_type.as_array())
            .map(|arr| arr.iter().any(|v| v.as_str() == Some("cdylib")))
            .unwrap_or(false);
        if !is_program_crate {
            anyhow::bail!("Not a program crate (missing cdylib crate-type)");
        }
        let package = cargo_toml.get("package").context("Failed to get package")?;
        let name = package.get("name").context("Failed to get package name")?;
        let name = Identifier::from(name.as_str().context("Failed to get package name")?);
        cargo_toml
            .get("dependencies")
            .context("Failed to get dependencies")?
            .get("solarium-program")
            .context("solarium-program is not a dependency")?;
        let public_key = Self::get_program_id_from_file(&root, name.to_string())?;
        Ok(Self {
            name,
            public_key,
            folder,
        })
    }
}
