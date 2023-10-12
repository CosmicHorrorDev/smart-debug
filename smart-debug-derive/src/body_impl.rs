use crate::{
    attr::{container, field},
    utils,
};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Result, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

pub fn impl_derive(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let name_lit_str = name.to_string();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let container_attrs = container::Attrs::parse(&input.attrs)?;
    let (body_expr, struct_kind) = match &input.data {
        syn::Data::Struct(body) => body_tt(&body.fields, &container_attrs.ignore)?,
        _ => todo!("Only structs are currently supported"),
    };

    let container_defaults = match container_attrs.ignore {
        None | Some(container::Ignore::Bare) => TokenStream::new(),
        Some(container::Ignore::Defaults) => quote! { let container_default = <#name>::default(); },
    };

    // TODO: dedupe logic from the match arms
    let debug_impl = match struct_kind {
        StructKind::NonTuple => {
            quote! {
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
            }
        }
        StructKind::Tuple => {
            quote! {
                impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        let mut debug = f.debug_tuple(#name_lit_str);
                        #container_defaults

                        #body_expr

                        debug.finish()
                    }
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

enum StructKind {
    NonTuple,
    Tuple,
}

fn body_tt(
    fields: &Fields,
    global_ignore: &Option<container::Ignore>,
) -> Result<(TokenStream, StructKind)> {
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
                            let has_interpol = utils::needs_formatting(&bare.value());

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
            let tokens = quote! { #( #formatted_fields )* };
            Ok((tokens, StructKind::NonTuple))
        }
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let formatted_fields = unnamed
                .iter()
                .enumerate()
                .map(|(field_num, field)| {
                    let field_name = syn::Index::from(field_num);
                    let field::Attrs {
                        bare_or_wrapper,
                        ignore: field_ignore,
                    } = field::Attrs::parse(&field.attrs)?;
                    let ignore = Ignore::new(global_ignore, field_ignore);
                    let cond = match ignore {
                        Ignore::No => quote! { false },
                        Ignore::Unconditional => quote! { true },
                        Ignore::Default => {
                            let ty = field.ty.to_owned();
                            quote! { self.#field_name == <#ty>::default() }
                        }
                        Ignore::DefaultGlobal => {
                            quote! { self.#field_name == container_default.#field_name }
                        }
                        Ignore::If(value) => quote! { self.#field_name == #value },
                        Ignore::Fn(value) => quote! { #value(&self.#field_name) },
                    };

                    let field_tokens = match bare_or_wrapper {
                        Some(field::BareOrWrapper::Bare(bare)) => {
                            let has_interpol = utils::needs_formatting(&bare.value());

                            // Use `format_args!()` if it's an interpolated str
                            if has_interpol {
                                quote! { ::std::format_args!(#bare, &self.#field_name) }
                            } else {
                                quote! { &#bare }
                            }
                        }
                        Some(field::BareOrWrapper::Wrapper(wrapper)) => {
                            quote! { #wrapper(&self.#field_name) }
                        }
                        None => quote! { self.#field_name },
                    };

                    let field_tokens = quote! {
                        if #cond {
                            debug.field(&::smart_debug::__IgnoredTupleStructField);
                        } else {
                            debug.field(&#field_tokens);
                        }
                    };
                    Ok(field_tokens)
                })
                .collect::<Result<Vec<_>>>()?;
            let tokens = quote! { #( #formatted_fields )* };
            Ok((tokens, StructKind::Tuple))
        }
        Fields::Unit => Ok((TokenStream::new(), StructKind::NonTuple)),
    }
}
