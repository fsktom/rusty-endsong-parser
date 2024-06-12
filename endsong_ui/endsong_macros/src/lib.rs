use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitInt};

#[proc_macro]
/// Generates match arms for creating space strings
pub fn generate_spaces_match(input: TokenStream) -> TokenStream {
    let n = parse_macro_input!(input as LitInt);
    let n = n.base10_parse::<usize>().unwrap();

    let arms = (1..=n).map(|i| {
        let spaces = " ".repeat(i);
        quote! { #i => #spaces, }
    });

    let expanded = quote! {
        match num {
            #( #arms )*
            _ => "",
        }
    };

    expanded.into()
}
