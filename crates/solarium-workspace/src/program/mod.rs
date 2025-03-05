use crate::prelude::*;
use ligen_ir::Identifier;
use solana_sdk::pubkey::Pubkey;

use crate::idl::Idl;
use crate::Workspace;

pub struct Program {
    pub name: Identifier,
    pub public_key: Pubkey,
}

impl Program {
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
}
