use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse2, ItemType, Token, Type};

use crate::fn_trait::FnTrait;

pub struct FactoryTypeAlias
{
    pub type_alias: ItemType,
    pub factory_interface: FnTrait,
    pub arg_types: Punctuated<Type, Token![,]>,
    pub return_type: Type,
}

impl Parse for FactoryTypeAlias
{
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let type_alias = input
            .parse::<ItemType>()
            .map_err(|_| input.error("Expected a type alias"))?;

        let aliased_fn_trait =
            parse2::<FnTrait>(type_alias.ty.as_ref().to_token_stream())?;

        Ok(Self {
            type_alias,
            factory_interface: aliased_fn_trait.clone(),
            arg_types: aliased_fn_trait.inputs,
            return_type: aliased_fn_trait.output,
        })
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use quote::{format_ident, quote};
    use syn::token::And;
    use syn::{Path, PathSegment, TypePath, TypeReference};

    use super::*;
    use crate::test_utils;

    #[test]
    fn can_parse() -> Result<(), Box<dyn Error>>
    {
        let input_args = quote! {
            type FooFactory = dyn Fn(String, &u32) -> Foo;
        };

        let factory_type_alias = parse2::<FactoryTypeAlias>(input_args)?;

        assert_eq!(
            factory_type_alias.arg_types,
            Punctuated::from_iter(vec![
                test_utils::create_type(test_utils::create_path(&[
                    test_utils::create_path_segment(format_ident!("String"), &[])
                ])),
                Type::Reference(TypeReference {
                    and_token: And::default(),
                    lifetime: None,
                    mutability: None,
                    elem: Box::new(test_utils::create_type(test_utils::create_path(&[
                        test_utils::create_path_segment(format_ident!("u32"), &[])
                    ])))
                })
            ])
        );

        Ok(())
    }
}
