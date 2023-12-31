use std::iter::FromIterator;

use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Attribute, Expr, Ident, LitStr, Token,
};

#[derive(Clone, Debug)]
pub enum Skip {
    Bare,
    Defaults,
}

#[derive(Clone, Debug, Default)]
pub struct Attrs {
    pub bare: Option<LitStr>,
    pub skip: Option<Skip>,
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
                    assert!(attrs.bare.is_none());
                }
                AttrName::Valueless(ValuelessName::Skip)
                | AttrName::Valueless(ValuelessName::SkipDefaults) => {
                    assert!(attrs.skip.is_none());
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
                            attrs.bare = Some(lit);
                        }
                    }
                }
                AttrName::Valueless(valueless) => match valueless {
                    ValuelessName::Skip => attrs.skip = Some(Skip::Bare),
                    ValuelessName::SkipDefaults => attrs.skip = Some(Skip::Defaults),
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
                    Err(_) => panic!("Failed parsing container expr"),
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
}

#[derive(Clone, Debug)]
pub enum ValuelessName {
    Skip,
    SkipDefaults,
}

impl AttrName {
    fn new(ident: Ident) -> Option<Self> {
        let name = match ident.to_string().as_str() {
            "skip" => Self::Valueless(ValuelessName::Skip),
            "skip_defaults" => Self::Valueless(ValuelessName::SkipDefaults),
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
