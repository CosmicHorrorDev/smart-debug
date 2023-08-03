use crate::attr::{container, field};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Result, DeriveInput, Fields, FieldsNamed};

pub fn impl_derive(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let name_lit_str = name.to_string();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let container_attrs = container::Attrs::parse(&input.attrs)?;
    let body_expr = match &input.data {
        syn::Data::Struct(body) => body_tt(&body.fields, &container_attrs.ignore)?,
        _ => panic!("Only structs are currently supported"),
    };

    let container_defaults = match container_attrs.ignore {
        None | Some(container::Ignore::Bare) => quote! {},
        Some(container::Ignore::Defaults) => quote! { let container_default = <#name>::default(); },
    };

    let debug_impl = quote! {
        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let mut debug = f.debug_struct(#name_lit_str);
                let mut field_was_ignored = false;
                #container_defaults

                #body_expr

                if field_was_ignored {
                    debug.finish_non_exhaustive()
                } else {
                    debug.finish()
                }
            }
        }
    };

    Ok(debug_impl)
}

enum Ignore {
    No,
    Unconditional,
    Default,
    DefaultGlobal,
    If(field::AttrValue),
    Fn(field::AttrValue),
}

impl Ignore {
    // local takes precedence over global
    fn new(global: &Option<container::Ignore>, local: Option<field::Ignore>) -> Self {
        match (global, local) {
            (_, Some(field::Ignore::No)) | (None, None) => Self::No,
            (Some(container::Ignore::Bare), None) | (_, Some(field::Ignore::Bare)) => {
                Self::Unconditional
            }
            (Some(container::Ignore::Defaults), None) => Self::DefaultGlobal,
            (_, Some(field::Ignore::Default)) => Self::Default,
            (_, Some(field::Ignore::If(value))) => Self::If(value),
            (_, Some(field::Ignore::Fn(value))) => Self::Fn(value),
        }
    }
}

fn body_tt(fields: &Fields, global_ignore: &Option<container::Ignore>) -> Result<TokenStream> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => {
            let formatted_fields = named
                .iter()
                .map(|field| {
                    let field_name = &field.ident;
                    let field::Attrs {
                        bare_or_wrapper,
                        ignore: field_ignore,
                    } = field::Attrs::parse(&field.attrs)?;
                    let ignore = Ignore::new(global_ignore, field_ignore);
                    let maybe_cond = match ignore {
                        Ignore::No => None,
                        Ignore::Unconditional => {
                            return Ok(quote! { field_was_ignored = true; });
                        }
                        Ignore::Default => {
                            let ty = field.ty.to_owned();
                            Some(quote! { self.#field_name == <#ty>::default() })
                        }
                        Ignore::DefaultGlobal => {
                            Some(quote! { self.#field_name == container_default.#field_name })
                        }
                        Ignore::If(value) => Some(quote! { self.#field_name == #value }),
                        Ignore::Fn(value) => Some(quote! { #value(&self.#field_name) }),
                    };

                    let field_tokens = match bare_or_wrapper {
                        Some(field::BareOrWrapper::Bare(bare)) => {
                            let bare_str = bare.value();
                            let mut was_interpol_start = false;
                            let mut char_it = bare_str.chars();
                            let has_interpol = loop {
                                let Some(curr) = char_it.next() else {
                                    break false;
                                };

                                let is_interpol_start = curr == '{' && !was_interpol_start;
                                if is_interpol_start {
                                    // Make sure the next char isn't escaping this one
                                    if char_it.clone().next() != Some('{') {
                                        break true;
                                    }
                                }

                                was_interpol_start = is_interpol_start;
                            };

                            // Use `format_args!()` if it's an interpolated str
                            if has_interpol {
                                quote! {
                                    ::std::format_args!(#bare, &self.#field_name)
                                }
                            } else {
                                quote! { &#bare }
                            }
                        }
                        Some(field::BareOrWrapper::Wrapper(wrapper)) => {
                            quote! { #wrapper(&self.#field_name) }
                        }
                        None => quote! { self.#field_name },
                    };

                    let field_name_str = field_name.as_ref().unwrap().to_string();
                    let field_tokens = match maybe_cond {
                        Some(cond_value) => {
                            quote! {
                                if #cond_value {
                                    field_was_ignored = true;
                                } else {
                                    debug.field(#field_name_str, &#field_tokens);
                                }
                            }
                        }
                        None => quote! {
                            debug.field(#field_name_str, &#field_tokens);
                        },
                    };
                    Ok(field_tokens)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                    #( #formatted_fields )*
            })
        }
        Fields::Unnamed(_) => todo!("Tuple structs are currently unsupported"),
        Fields::Unit => todo!("Unit structs are currently unsupported"),
    }
}
