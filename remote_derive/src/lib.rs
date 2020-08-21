// use proc_macro2::{Ident, Literal};
// use quote::{quote, quote_spanned};
// use syn::spanned::Spanned;
// use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn remote(
    _: proc_macro::TokenStream,
    _input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // let input = parse_macro_input!(input as DeriveInput);
    // let output = quote!( #( input )* );
    // println!("{}", output.to_string());
    // proc_macro::TokenStream::from(output)
    panic!()
}
