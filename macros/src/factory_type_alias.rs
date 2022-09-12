use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse, ItemType, Token, Type};

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
        let type_alias = match input.parse::<ItemType>() {
            Ok(type_alias) => Ok(type_alias),
            Err(_) => Err(input.error("Expected a type alias")),
        }?;

        let aliased_fn_trait =
            parse::<FnTrait>(type_alias.ty.as_ref().to_token_stream().into())?;

        Ok(Self {
            type_alias,
            factory_interface: aliased_fn_trait.clone(),
            arg_types: aliased_fn_trait.inputs,
            return_type: aliased_fn_trait.output,
        })
    }
}
