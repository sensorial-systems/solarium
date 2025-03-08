use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use solarium_workspace::Workspace;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    Idl
}

#[tokio::main]
async fn main() -> Result<()> {
    let workspace = Workspace::current()?;
    let cli = Cli::parse();

    match cli.command {
        Commands::Idl => {
            for program in &workspace.programs {
                let idl = program.idl().await?;
                idl.save(&workspace).context("Failed to save IDL")?;
            }
        }
        Commands::Build => {
            workspace.build().await?;
        }
        Commands::Dev => {
            workspace.dev().await?.wait().await?;
        }
        Commands::Test { detach } => {
            if detach {
                println!("Detaching test validator...");
                workspace.test().await?.wait().await?;
            } else {
                workspace.test().await?.kill().await?;
            }
        }
        Commands::Programs => {
            println!("Programs ({}):", workspace.programs.len());
            for program in workspace.programs {
                println!("{} ({})", program.name, program.public_key);
            }
        }
        Commands::Deploy { program } => {
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
