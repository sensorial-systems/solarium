use crate::prelude::*;

use ligen::prelude::{ConfigSet, Generator};
use ligen_rust::generator::Config;
use solarium_workspace::Workspace;

pub fn process(input: proc_macro::TokenStream) -> Result<proc_macro2::TokenStream> {
    let input = syn::parse::<syn::LitStr>(input)?;
    let program_name = &input;

    let workspace = Workspace::current()?;
    let program = workspace.program(program_name.value()).context(format!("Couldn't find program {}", program_name.value()))?;
    let program_idl = program.idl().context(format!("Failed to get program IDL for {}", program_name.value()))?;

    let mut config = Config::default();
    let program_name_str = program_name.value();
    config.set("program-name", program_name_str.clone());
    // Provide crate path (snake_case) for generated code to import program types
    let program_crate = program_name_str.replace('-', "_");
    config.set("program-crate", program_crate);
    let module = solarium_rust_client_generator::ModuleGenerator::default().generate(&program_idl.idl.root_module, &config)?;
    
    let content = module.content.map(|(_, items)| items).unwrap_or_default();

    let output = quote! {
        #(#content)*
    };

    Ok(output)
}
