use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{Token, Type};

use crate::macro_flag::MacroFlag;
use crate::util::iterator_ext::IteratorExt;

pub const FACTORY_MACRO_FLAGS: &[&str] = &["threadsafe", "async"];

pub struct DeclareDefaultFactoryMacroArgs
{
    pub interface: Type,
    pub flags: Punctuated<MacroFlag, Token![,]>,
}

impl Parse for DeclareDefaultFactoryMacroArgs
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self>
    {
        let interface = input.parse()?;

        if !input.peek(Token![,]) {
            return Ok(Self {
                interface,
                flags: Punctuated::new(),
            });
        }

        input.parse::<Token![,]>()?;

        let flags = Punctuated::<MacroFlag, Token![,]>::parse_terminated(input)?;

        for flag in &flags {
            let flag_str = flag.flag.to_string();

            if !FACTORY_MACRO_FLAGS.contains(&flag_str.as_str()) {
                return Err(input.error(format!(
                    "Unknown flag '{}'. Expected one of [ {} ]",
                    flag_str,
                    FACTORY_MACRO_FLAGS.join(",")
                )));
            }
        }

        let flag_names = flags
            .iter()
            .map(|flag| flag.flag.to_string())
            .collect::<Vec<_>>();

        if let Some(dupe_flag_name) = flag_names.iter().find_duplicate() {
            return Err(input.error(format!("Duplicate flag '{dupe_flag_name}'")));
        }

        Ok(Self { interface, flags })
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use proc_macro2::Span;
    use quote::{format_ident, quote};
    use syn::token::Dyn;
    use syn::{
        parse2,
        LitBool,
        Path,
        PathArguments,
        PathSegment,
        TraitBound,
        TraitBoundModifier,
        Type,
        TypeParamBound,
        TypeTraitObject,
    };

    use super::*;

    #[test]
    fn can_parse_with_interface_only() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            dyn IFoo
        };

        let dec_def_fac_args = parse2::<DeclareDefaultFactoryMacroArgs>(input_args)?;

        assert_eq!(
            dec_def_fac_args.interface,
            Type::TraitObject(TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![PathSegment {
                            ident: format_ident!("IFoo"),
                            arguments: PathArguments::None
                        }])
                    }
                })])
            })
        );

        assert!(dec_def_fac_args.flags.is_empty());

        Ok(())
    }

    #[test]
    fn can_parse_with_interface_and_single_flag() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            dyn IBar, threadsafe = true
        };

        let dec_def_fac_args = parse2::<DeclareDefaultFactoryMacroArgs>(input_args)?;

        assert_eq!(
            dec_def_fac_args.interface,
            Type::TraitObject(TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![PathSegment {
                            ident: format_ident!("IBar"),
                            arguments: PathArguments::None
                        }])
                    }
                })])
            })
        );

        assert_eq!(
            dec_def_fac_args.flags,
            Punctuated::from_iter(vec![MacroFlag {
                flag: format_ident!("threadsafe"),
                is_on: LitBool::new(true, Span::call_site())
            }])
        );

        Ok(())
    }

    #[test]
    fn can_parse_with_interface_and_multiple_flags() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            dyn IBar, threadsafe = true, async = false
        };

        let dec_def_fac_args = parse2::<DeclareDefaultFactoryMacroArgs>(input_args)?;

        assert_eq!(
            dec_def_fac_args.interface,
            Type::TraitObject(TypeTraitObject {
                dyn_token: Some(Dyn::default()),
                bounds: Punctuated::from_iter(vec![TypeParamBound::Trait(TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter(vec![PathSegment {
                            ident: format_ident!("IBar"),
                            arguments: PathArguments::None
                        }])
                    }
                })])
            })
        );

        assert_eq!(
            dec_def_fac_args.flags,
            Punctuated::from_iter(vec![
                MacroFlag {
                    flag: format_ident!("threadsafe"),
                    is_on: LitBool::new(true, Span::call_site())
                },
                MacroFlag {
                    flag: format_ident!("async"),
                    is_on: LitBool::new(false, Span::call_site())
                }
            ])
        );

        Ok(())
    }

    #[test]
    fn cannot_parse_with_interface_and_invalid_flag()
    {
        let input_args = quote! {
            dyn IBar, async = true, foo = false
        };

        assert!(matches!(
            parse2::<DeclareDefaultFactoryMacroArgs>(input_args),
            Err(_)
        ));
    }

    #[test]
    fn cannot_parse_with_interface_and_duplicate_flag()
    {
        assert!(matches!(
            parse2::<DeclareDefaultFactoryMacroArgs>(quote! {
                dyn IBar, async = true, threadsafe = false, async = true
            }),
            Err(_)
        ));

        assert!(matches!(
            parse2::<DeclareDefaultFactoryMacroArgs>(quote! {
                dyn IBar, async = true, threadsafe = false, async = false
            }),
            Err(_)
        ));
    }
}
