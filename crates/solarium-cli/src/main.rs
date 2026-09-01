use std::path::Path;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use solarium_workspace::{IdlType, Workspace};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// New a new program
    New {
        /// The name of the program
        name: String,
    },
    /// Build the programs
    Build,
    /// Test the programs
    Test {
        #[arg(short, long)]
        detach: bool,
    },
    /// List all programs
    Programs,
    /// Start the local validator and deploy the programs
    Dev,
    /// Deploy the programs
    Deploy {
        /// The program to deploy
        program: Option<String>,
    },
    /// Generate the IDL for the program
    Idl,
}

#[tokio::main]
async fn main() -> Result<()> {
    let workspace = Workspace::current();
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => {
            if let Ok(workspace) = workspace {
                let path = std::env::current_dir().context("Failed to get current directory")?;
                workspace.new_program(&name, &path)?;
            } else {
                let workspace = Workspace::new(&name)?;
                let path = Path::new(&name).join("programs");
                workspace.new_program(&name, &path)?;
            }
        }
        Commands::Idl => {
            let workspace = workspace?;
            for program in &workspace.programs {
                let idl = program.idl()?;
                idl.save_as(&workspace, IdlType::Anchor)?;
            }
        }
        Commands::Build => {
            workspace?.build().await?;
        }
        Commands::Dev => {
            workspace?.dev().await?.wait().await?;
        }
        Commands::Test { detach } => {
            let workspace = workspace?;
            if detach {
                workspace.test().await?.wait().await?;
            } else {
                workspace.test().await?.kill().await?;
            }
        }
        Commands::Programs => {
            let workspace = workspace?;
            println!("Programs ({}):", workspace.programs.len());
            for program in &workspace.programs {
                println!("{} ({})", program.name, program.public_key);
            }
        }
        Commands::Deploy { program } => {
            let workspace = workspace?;
            if let Some(program) = program {
                let program = workspace.program(program).context("program not found")?;
                program.deploy(&workspace).await?;
            } else {
                workspace.deploy().await?;
            }
        }
    }

    Ok(())
}
