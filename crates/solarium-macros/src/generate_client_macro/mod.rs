use crate::prelude::*;

use ligen::prelude::{ConfigSet, Generator};
use ligen_rust::generator::Config;
use solarium_workspace::{Idl, Workspace};
use std::path::PathBuf;
use std::process::Command;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Token};

struct GenerateClientInput {
    program_name: LitStr,
    program_address: Option<LitStr>,
}

impl Parse for GenerateClientInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let program_name = input.parse()?;
        let program_address = if input.is_empty() {
            None
        } else {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        };

        if !input.is_empty() {
            return Err(input.error("expected a program name and optional program address"));
        }

        Ok(Self {
            program_name,
            program_address,
        })
    }
}

pub fn process(input: proc_macro::TokenStream) -> Result<proc_macro2::TokenStream> {
    let input = syn::parse::<GenerateClientInput>(input)?;
    let program_name = &input.program_name;

    let folder = program_folder(&program_name.value())?;
    let program_idl = Idl::program_interface(&folder).context(format!(
        "Failed to get program IDL for {}",
        program_name.value()
    ))?;

    let mut config = Config::default();
    let program_name_str = program_name.value();
    config.set("program-name", program_name_str.clone());
    if let Some(program_address) = input.program_address {
        config.set("program-address", program_address.value());
    }
    let program_crate = program_name_str.replace('-', "_");
    config.set("program-crate", program_crate);
    let module = solarium_rust_client_generator::ModuleGenerator::default()
        .generate(&program_idl.idl.root_module, &config)?;

    let content = module.content.map(|(_, items)| items).unwrap_or_default();

    let output = quote! {
        #(#content)*
    };

    Ok(output)
}

/// Where the program crate `name` is. A program in the same tree as the crate asking is found the
/// way the CLI finds it; one the crate only depends on — a git or registry dependency, somewhere
/// under Cargo's home — is found where Cargo put it.
fn program_folder(name: &str) -> Result<PathBuf> {
    if let Ok(workspace) = Workspace::current() {
        if let Some(program) = workspace.program(name) {
            return Ok(program.folder.clone());
        }
    }
    dependency_folder(name).context(format!(
        "Couldn't find program {name}: it is neither in this workspace nor a dependency of this crate"
    ))
}

/// The folder of the package `name` in the dependency graph of the crate being compiled.
///
/// Offline first: by the time a macro expands, everything the build needs is on disk, and
/// metadata that can reach the network waits on it for nothing. Metadata covers every target and
/// dev-dependency, though, so offline can miss a package this build never needed; then it is
/// asked again as it would be from the command line.
fn dependency_folder(name: &str) -> Result<PathBuf> {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR is not set")?;
    let manifest = PathBuf::from(manifest_dir).join("Cargo.toml");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let metadata = |offline: bool| -> Result<serde_json::Value> {
        let mut command = Command::new(&cargo);
        command
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .arg("--manifest-path")
            .arg(&manifest);
        if offline {
            command.arg("--offline");
        }
        let output = command.output().context("Failed to run cargo metadata")?;
        if !output.status.success() {
            anyhow::bail!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(serde_json::from_slice(&output.stdout)?)
    };
    let metadata = metadata(true).or_else(|_| metadata(false))?;
    let manifest_path = metadata["packages"]
        .as_array()
        .context("cargo metadata listed no packages")?
        .iter()
        .find(|package| package["name"].as_str() == Some(name))
        .and_then(|package| package["manifest_path"].as_str())
        .context(format!("{name} is not in the dependency graph"))?;
    Ok(PathBuf::from(manifest_path)
        .parent()
        .context("a manifest has no folder")?
        .to_path_buf())
}
