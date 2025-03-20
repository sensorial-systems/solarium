use std::path::PathBuf;

use anyhow::{Context, Result};
use ligen_ir::Identifier;

use crate::Program;

pub struct Workspace {
    pub root: PathBuf,
    pub programs: Vec<Program>,
}

impl Workspace {
    pub fn current() -> Result<Self> {
        let mut programs = vec![];
        let root = project_root::get_project_root().context("Failed to get project root")?;
        let deploy_path = root.join("target").join("deploy");
        if let Ok(entries) = std::fs::read_dir(deploy_path).context("Failed to read deploy path") {
            for entry in entries {
                let entry = entry.context("Failed to read entry")?;
                let path = entry.path();
            if let Ok(program) = Program::try_from(&root, path) {
                programs.push(program);
                }
            }
        }
        
        Ok(Workspace { root, programs })
    }

    pub fn new_program(&self, name: impl AsRef<str>) -> Result<()> {
        std::process::Command::new("cargo")
            .arg("new")
            .arg(name.as_ref())
            .arg("--lib")
            .status()
            .context("Failed to create new program")?;
        let program_name = Identifier::new(name.as_ref()).to_pascal_case();
        let lib_content = format!(include_str!("templates/program/src/lib.rs.template"), program_name = program_name);
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        let path = current_dir.join(name.as_ref()).join("src/lib.rs");
        std::fs::write(&path, lib_content)
            .context(format!("Failed to write {}", path.display()))?;
        let cargo_toml_path = current_dir.join(name.as_ref()).join("Cargo.toml");
        let mut cargo_toml_content = std::fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;
        cargo_toml_content = cargo_toml_content.replace("[dependencies]\n", "[dependencies]\nsolarium.workspace = true\n");
        std::fs::write(&cargo_toml_path, cargo_toml_content).context("Failed to write Cargo.toml")?;
        Ok(())
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