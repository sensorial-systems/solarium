use clap::{Parser, Subcommand};
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => {
            let status = std::process::Command::new("cargo")
                .arg("build-sbf")
                .status()
                .context("failed to run cargo build")?;

            if !status.success() {
                anyhow::bail!("cargo build-sbf failed");
            }
        }
    }

    Ok(())
}
