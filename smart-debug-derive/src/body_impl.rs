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

    let container::Attrs {
        ignore: container_ignore,
        bare: container_bare,
    } = container::Attrs::parse(&input.attrs)?;

    let fn_body = match container_bare {
        Some(lit_str) => quote! { f.write_str(#lit_str) },
        None => {
            let (body_expr, struct_kind) = match &input.data {
                syn::Data::Struct(body) => body_tt(&body.fields, &container_ignore)?,
                _ => todo!("Only structs are currently supported"),
            };

            let container_defaults = match container_ignore {
                None | Some(container::Ignore::Bare) => TokenStream::new(),
                Some(container::Ignore::Defaults) => {
                    quote! { let container_default = <#name>::default(); }
                }
            };

            let formatting_code = match struct_kind {
                StructKind::NonTuple => {
                    quote! {
                        let mut debug = f.debug_struct(#name_lit_str);
                        let mut field_was_ignored = false;
                        #body_expr
                        if field_was_ignored {
                            debug.finish_non_exhaustive()
                        } else {
                            debug.finish()
                        }
                    }
                }
                StructKind::Tuple => {
                    quote! {
                        let mut debug = f.debug_tuple(#name_lit_str);
                        #body_expr
                        debug.finish()
                    }
                }
            };

            quote! {
                #container_defaults
                #formatting_code
            }
        }
    };

    let debug_impl = quote! {
        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #fn_body
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

// TODO: the generated code here could be better by avoiding some code in different situations
// TODO: the regular struct and tuple struct arms could share more code
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
                            let args = if has_interpol {
                                quote! { #bare, &self.#field_name }
                            } else {
                                quote! { #bare }
                            };
                            quote! {
                                ::smart_debug::internal::__LiteralField(::std::format_args!(#args))
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
                            let args = if has_interpol {
                                quote! { #bare, &self.#field_name }
                            } else {
                                quote! { #bare }
                            };
                            quote! {
                                ::smart_debug::internal::__LiteralField(::std::format_args!(#args))
                            }
                        }
                        Some(field::BareOrWrapper::Wrapper(wrapper)) => {
                            quote! { #wrapper(&self.#field_name) }
                        }
                        None => quote! { self.#field_name },
                    };

                    let field_tokens = quote! {
                        if #cond {
                            debug.field(&::smart_debug::internal::__IgnoredTupleField);
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
