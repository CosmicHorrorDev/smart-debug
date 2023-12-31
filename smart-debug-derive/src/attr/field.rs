use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, Ident, LitStr, Token,
};

// TODO: make bare and wrapper exclusive
#[derive(Clone, Debug, Default)]
pub struct Attrs {
    pub bare_or_wrapper: Option<BareOrWrapper>,
    pub skip: Option<Skip>,
}

#[derive(Clone, Debug)]
pub enum BareOrWrapper {
    Bare(LitStr),
    Wrapper(AttrValue),
}

#[derive(Clone, Debug)]
pub enum Skip {
    No,
    Bare,
    Default,
    If(AttrValue),
    Fn(AttrValue),
}

impl Attrs {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut parsed = Vec::new();
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("debug")) {
            for attr in attr.parse_args_with(Punctuated::<Attr, Token![,]>::parse_terminated)? {
                parsed.push(attr);
            }
        }

        Self::try_from(parsed)
    }
}

impl TryFrom<Vec<Attr>> for Attrs {
    type Error = syn::Error;

    fn try_from(unstructured: Vec<Attr>) -> Result<Self, Self::Error> {
        let mut attrs = Self::default();

        for Attr { name, value } in unstructured {
            // Validate
            match name {
                AttrName::Valuefull(ValuefullName::Bare) => {
                    assert!(attrs.bare_or_wrapper.is_none());
                }
                AttrName::Valueless(
                    ValuelessName::Skip | ValuelessName::SkipDefault | ValuelessName::NoSkip,
                )
                | AttrName::Valuefull(ValuefullName::SkipFn | ValuefullName::SkipIf) => {
                    assert!(attrs.skip.is_none());
                }
                AttrName::Valuefull(ValuefullName::Wrapper) => {
                    assert!(attrs.bare_or_wrapper.is_none());
                }
            }

            // Parse
            match name {
                AttrName::Valuefull(valuefull) => {
                    let value = value.unwrap();
                    match valuefull {
                        ValuefullName::Bare => {
                            let AttrValue::LitStr(lit) = value else {
                                unreachable!()
                            };
                            attrs.bare_or_wrapper = Some(BareOrWrapper::Bare(lit));
                        }
                        ValuefullName::SkipFn => attrs.skip = Some(Skip::Fn(value)),
                        ValuefullName::SkipIf => attrs.skip = Some(Skip::If(value)),
                        ValuefullName::Wrapper => {
                            attrs.bare_or_wrapper = Some(BareOrWrapper::Wrapper(value));
                        }
                    }
                }
                AttrName::Valueless(valueless) => match valueless {
                    ValuelessName::Skip => attrs.skip = Some(Skip::Bare),
                    ValuelessName::SkipDefault => attrs.skip = Some(Skip::Default),
                    ValuelessName::NoSkip => attrs.skip = Some(Skip::No),
                },
            }
        }

        Ok(attrs)
    }
}

#[derive(Clone, Debug)]
pub struct Attr {
    pub name: AttrName,
    pub value: Option<AttrValue>,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse::<syn::LitStr>() {
            return Ok(Self {
                name: AttrName::Valuefull(ValuefullName::Bare),
                value: Some(AttrValue::LitStr(lit)),
            });
        }

        let ident: Ident = input.parse()?;
        let name = match AttrName::new(ident) {
            Some(name) => name,
            None => todo!(),
        };

        let value = if input.peek(Token![=]) {
            // `name = value` attributes.
            // TODO: vv
            let _assign_token = input.parse::<Token![=]>()?; // skip '='
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Some(AttrValue::LitStr(lit))
            } else {
                match input.parse::<Expr>() {
                    Ok(expr) => Some(AttrValue::Expr(expr)),
                    Err(_) => panic!("Failed parsing field expr"),
                }
            }
        } else if input.peek(syn::token::Paren) {
            // TODO: are we going to support these?
            // `name(...)` attributes.
            let nested;
            parenthesized!(nested in input);

            let method_args: Punctuated<_, _> = nested.parse_terminated(Expr::parse, Token![,])?;
            Some(AttrValue::Call(Vec::from_iter(method_args)))
        } else {
            None
        };

        Ok(Self { name, value })
    }
}

#[derive(Clone, Debug)]
pub enum AttrName {
    Valuefull(ValuefullName),
    Valueless(ValuelessName),
}

#[derive(Clone, Debug)]
pub enum ValuefullName {
    Bare,
    SkipFn,
    SkipIf,
    Wrapper,
}

#[derive(Clone, Debug)]
pub enum ValuelessName {
    NoSkip,
    Skip,
    SkipDefault,
}

impl AttrName {
    fn new(ident: Ident) -> Option<Self> {
        let name = match ident.to_string().as_str() {
            "no_skip" => Self::Valueless(ValuelessName::NoSkip),
            "skip_default" => Self::Valueless(ValuelessName::SkipDefault),
            "skip" => Self::Valueless(ValuelessName::Skip),
            "skip_fn" => Self::Valuefull(ValuefullName::SkipFn),
            "skip_if" => Self::Valuefull(ValuefullName::SkipIf),
            "wrapper" => Self::Valuefull(ValuefullName::Wrapper),
            _ => return None,
        };

        Some(name)
    }
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AttrValue {
    LitStr(LitStr),
    Expr(Expr),
    Call(Vec<Expr>),
}

impl ToTokens for AttrValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::LitStr(t) => t.to_tokens(tokens),
            Self::Expr(t) => t.to_tokens(tokens),
            Self::Call(t) => {
                let t = quote!(#(#t),*);
                t.to_tokens(tokens)
            }
        }
    }
}
