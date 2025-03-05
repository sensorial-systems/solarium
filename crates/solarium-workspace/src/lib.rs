use std::path::PathBuf;

use anyhow::{Context, Result};
use ligen_ir::Identifier;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::{EncodableKey, Signer}};

pub struct Program {
    pub name: Identifier,
    pub public_key: Pubkey,
}

impl Program {
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
}

pub struct Workspace {
    pub root: PathBuf,
    pub programs: Vec<Program>,
}

impl Workspace {
    pub fn current() -> Result<Self> {
        let mut programs = vec![];
        let root = project_root::get_project_root().context("Failed to get project root")?;
        let deploy_path = root.join("target").join("deploy");
        for entry in std::fs::read_dir(deploy_path).context("Failed to read deploy path")? {
            let entry = entry.context("Failed to read entry")?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") && path.file_name().unwrap().to_string_lossy().to_string().contains("-keypair") {
                let keypair = Keypair::read_from_file(&path).map_err(|e| anyhow::anyhow!("Failed to load keypair: {}", e))?;
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let name = name.replace("-keypair.json", "");
                let name = Identifier::from(name);
                let name = name.to_kebab_case();
                let public_key = keypair.pubkey();
                programs.push(Program { name, public_key });
            }
        }
        
        Ok(Workspace { root, programs })
    }

    pub fn program(&self, name: impl AsRef<str>) -> Option<&Program> {
        self.programs.iter().find(|p| p.name == name.as_ref())
    }

    pub async fn dev(&self) -> Result<tokio::process::Child> {
        let test_ledger = self.root.join(".test-ledger");
        let solana_test_validator = tokio::process::Command::new("solana-test-validator")
            .arg("--reset")
            .arg("--ledger")
            .arg(test_ledger.to_string_lossy().as_ref())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let mut attempts = 3;
        while let Err(_e) = self.deploy().await {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            attempts -= 1;
            if attempts == 0 {
                panic!("failed to deploy programs");
            }
        }

        Ok(solana_test_validator)
    }

    pub async fn test(&self) -> Result<tokio::process::Child> {
        let child = self.dev().await?;
        let output = tokio::process::Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--nocapture")
            .status()
            .await
            .context("failed to run cargo test")?;

        if !output.success() {
            anyhow::bail!("cargo test failed");
        }

        Ok(child)
    }

    pub async fn build(&self) -> Result<()> {
        let status = tokio::process::Command::new("cargo")
        .arg("build-sbf")
        .status()
        .await
        .context("failed to run cargo build")?;

        if !status.success() {
            anyhow::bail!("cargo build-sbf failed");
        }

        Ok(())
    }

    pub async fn deploy(&self) -> Result<()> {
        for program in self.programs.iter() {
            program.deploy(self).await?;
        }
        Ok(())
    }
}