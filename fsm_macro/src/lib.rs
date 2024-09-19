// Higher recursion limit for quote
#![recursion_limit = "512"]

extern crate proc_macro;

use crate::fsm::machine::Machine;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod fsm;

/// Generate the declaratively described state machine diagram.
///
/// See the main crate documentation for more details.
#[proc_macro]
pub fn fsm(input: TokenStream) -> TokenStream {
    let fsm: Machine = parse_macro_input!(input as Machine);
    
    let expanded = quote!(#fsm);
//    println!("{}", expanded);

    expanded.into()
}
