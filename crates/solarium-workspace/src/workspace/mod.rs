use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ligen_ir::Identifier;
use ligen_rust_parser::cargo::Cargo;

use crate::Program;

#[derive(Debug)]
pub struct Workspace {
    pub root: PathBuf,
    pub programs: Vec<Program>,
}

impl Workspace {
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut programs = vec![];

        for directory in walkdir::WalkDir::new(&root) {
            let entry = directory.context("Failed to read directory")?;
            let entry = entry.path();
            if entry.is_file() && entry.file_name().map(|file_name| file_name.to_str() == Some("Cargo.toml")).unwrap_or(false) {
                let folder = entry.parent().context("Failed to get parent directory")?;
                if let Ok(program) = Program::try_from(&root, folder.to_path_buf()) {
                    programs.push(program);
                }
            }
        }
        
        Ok(Workspace { root, programs })
    }

    pub fn new(name: impl AsRef<str>) -> Result<Self> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        let workspace_path = current_dir.join(name.as_ref());
        
        std::process::Command::new("git")
            .arg("init")
            .arg(&workspace_path)
            .args(["-b", "main"])
            .status()
            .context("Failed to initialize git repository")?;

        std::fs::write(workspace_path.join(".gitignore"), include_str!("templates/workspace/.gitignore.template")).context("Failed to write .gitignore")?;

        let cargo_toml_path = workspace_path.join("Cargo.toml");
        let cargo_toml_content = include_str!("templates/workspace/Cargo.toml.template").replace("{solarium_version}", env!("CARGO_PKG_VERSION"));
        std::fs::write(&cargo_toml_path, cargo_toml_content).context("Failed to write Cargo.toml")?;
        Self::from_root(&workspace_path)
    }

    pub fn current() -> Result<Self> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        let root = Cargo::get_project_root_from_path(&current_dir).context("Failed to get project root")?;
        Self::from_root(&root)
    }

    pub fn new_program(&self, name: impl AsRef<str>, path: impl AsRef<Path>) -> Result<()> {
        let root = path.as_ref().join(name.as_ref());

        // Create new program
        std::process::Command::new("cargo")
            .arg("new")
            .arg("--lib")
            .arg(&root)
            .status()
            .context("Failed to create new program")?;
        let program_name = Identifier::new(name.as_ref()).to_pascal_case();
        
        // Write lib.rs
        let lib_content = format!(include_str!("templates/program/src/lib.rs.template"), program_name = program_name);
        let lib_path = root.join("src/lib.rs");
        std::fs::write(&lib_path, lib_content)
            .context(format!("Failed to write {}", lib_path.display()))?;
        
        // Write Cargo.toml
        let cargo_toml_path = root.join("Cargo.toml");
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
        self.build().await?;
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