mod program_id_macro;
mod program_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

/// Fetches the Program ID in the current workspace.
#[proc_macro]
pub fn program_id(input: TokenStream) -> TokenStream {
    let program_name = parse_macro_input!(input as LitStr);
    let program_name = program_name.value();
    program_id_macro::process(&program_name).expect("Failed to generate program ID").into()
}

/// Fetches the Program ID of the current crate.
#[proc_macro]
pub fn current_program_id(_input: TokenStream) -> TokenStream {
    let current_program_id = std::env::var("CARGO_PKG_NAME").expect("Failed to get current program ID");
    program_id_macro::process(&current_program_id).expect("Failed to generate program ID").into()
}


#[proc_macro_attribute]
pub fn program(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemImpl);
    program_macro::process(input).into()
}
