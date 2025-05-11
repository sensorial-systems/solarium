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
    config.set("program-name", program_name.value());
    let module = solarium_rust_client_generator::ModuleGenerator::default().generate(&program_idl.idl.root_module, &config)?;
    
    let content = module.content.map(|(_, items)| items).unwrap_or_default();

    let output = quote! {
        #(#content)*
    };

    Ok(output)
}
