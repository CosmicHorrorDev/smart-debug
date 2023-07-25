#![doc = include_str!("../README.md")]

use syn::{parse_macro_input, DeriveInput};

mod attr;
mod body_impl;

/// Derive macro for deriving [`Debug`] with easier customization
#[proc_macro_derive(SmartDebug, attributes(debug))]
pub fn derive_smart_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match body_impl::impl_derive(&input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
