use std::hash::Hash;

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Lit, LitBool, Token};

use crate::util::error::diagnostic_error_enum;

#[derive(Debug, Clone)]
pub struct MacroFlag
{
    pub name: Ident,
    pub value: MacroFlagValue,
}

impl MacroFlag
{
    pub fn new_off(flag: &str) -> Self
    {
        Self {
            name: Ident::new(flag, Span::call_site()),
            value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                false,
                Span::call_site(),
            ))),
        }
    }

    pub fn name(&self) -> &Ident
    {
        &self.name
    }

    pub fn get_bool(&self) -> Result<bool, MacroFlagError>
    {
        if let MacroFlagValue::Literal(Lit::Bool(lit_bool)) = &self.value {
            return Ok(lit_bool.value);
        }

        Err(MacroFlagError::UnexpectedValueKind {
            expected: "boolean literal",
            value_span: self.value.span(),
        })
    }

    pub fn get_ident(&self) -> Result<Ident, MacroFlagError>
    {
        if let MacroFlagValue::Identifier(ident) = &self.value {
            return Ok(ident.clone());
        }

        Err(MacroFlagError::UnexpectedValueKind {
            expected: "identifier",
            value_span: self.value.span(),
        })
    }
}

impl Parse for MacroFlag
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let name = input.parse::<Ident>()?;

        input.parse::<Token![=]>()?;

        let value: MacroFlagValue = input.parse()?;

        Ok(Self { name, value })
    }
}

impl PartialEq for MacroFlag
{
    fn eq(&self, other: &Self) -> bool
    {
        self.name == other.name
    }
}

impl Eq for MacroFlag {}

impl Hash for MacroFlag
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H)
    {
        self.name.hash(state);
    }
}

diagnostic_error_enum! {
pub enum MacroFlagError {
    #[error("Expected a {expected}"), span = value_span]
    UnexpectedValueKind {
        expected: &'static str,
        value_span: Span
    },
}
}

#[derive(Debug, Clone)]
pub enum MacroFlagValue
{
    Literal(Lit),
    Identifier(Ident),
}

impl MacroFlagValue
{
    fn span(&self) -> Span
    {
        match self {
            Self::Literal(lit) => lit.span(),
            Self::Identifier(ident) => ident.span(),
        }
    }
}

impl Parse for MacroFlagValue
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        if let Ok(lit) = input.parse::<Lit>() {
            return Ok(Self::Literal(lit));
        };

        input.parse::<Ident>().map(Self::Identifier).map_err(|err| {
            syn::Error::new(err.span(), "Expected a literal or a identifier")
        })
    }
}

#[cfg(test)]
mod tests
{
    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::{parse2, LitInt, LitStr};

    use super::*;

    #[test]
    fn parse_macro_flag_literal_works()
    {
        assert_eq!(
            parse2::<MacroFlag>(quote! {
                more = true
            })
            .expect("Expected Ok"),
            MacroFlag {
                name: format_ident!("more"),
                value: MacroFlagValue::Literal(Lit::Bool(LitBool::new(
                    true,
                    Span::call_site()
                )))
            }
        );

        assert_eq!(
            parse2::<MacroFlag>(quote! {
                guitarist = "John Norum"
            })
            .expect("Expected Ok"),
            MacroFlag {
                name: format_ident!("guitarist"),
                value: MacroFlagValue::Literal(Lit::Str(LitStr::new(
                    "John Norum",
                    Span::call_site()
                )))
            }
        );
    }

    #[test]
    fn parse_macro_flag_identifier_works()
    {
        assert_eq!(
            parse2::<MacroFlag>(quote! {
                error = FooError
            })
            .expect("Expected Ok"),
            MacroFlag {
                name: format_ident!("error"),
                value: MacroFlagValue::Identifier(Ident::new(
                    "FooError",
                    Span::call_site()
                ))
            }
        );
    }

    #[test]
    fn get_bool_works()
    {
        assert!(
            // Formatting is weird without this comment
            MacroFlag {
                name: format_ident!("rocked_the_night"),
                value: MacroFlagValue::Literal(Lit::Bool(LitBool {
                    value: true,
                    span: Span::call_site()
                }))
            }
            .get_bool()
            .expect("Expected Ok")
        );

        assert!(
            // Formatting is weird without this comment
            MacroFlag {
                name: format_ident!("vocalist"),
                value: MacroFlagValue::Literal(Lit::Str(LitStr::new(
                    "Joey Tempest",
                    Span::call_site()
                )))
            }
            .get_bool()
            .is_err()
        );
    }

    #[test]
    fn get_ident_works()
    {
        assert_eq!(
            MacroFlag {
                name: format_ident!("formed_in"),
                value: MacroFlagValue::Identifier(format_ident!("upplands_vasby"))
            }
            .get_ident()
            .expect("Expected Ok"),
            "upplands_vasby"
        );

        assert!(
            // Formatting is weird without this comment
            MacroFlag {
                name: format_ident!("formed"),
                value: MacroFlagValue::Literal(Lit::Int(LitInt::new(
                    "1979",
                    Span::call_site()
                )))
            }
            .get_ident()
            .is_err()
        );
    }

    #[test]
    fn parse_with_invalid_name_fails()
    {
        assert!(
            // Formatting is weird without this comment
            parse2::<MacroFlag>(quote! {
                10 = false
            })
            .is_err()
        );
    }

    #[test]
    fn parse_with_invalid_value_fails()
    {
        assert!(
            // Formatting is weird without this comment
            parse2::<MacroFlag>(quote! {
                years_active = ["1979-1992", "1999", "2003–present"]
            })
            .is_err()
        );
    }
}
