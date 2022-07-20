use syn::parse::{Parse, ParseStream};
use syn::{GenericArgument, ItemType, Path, Type, TypeParamBound, TypeTuple};

pub struct FactoryTypeAlias
{
    pub type_alias: ItemType,
    pub factory_interface: Path,
    pub arg_types: TypeTuple,
    pub return_type: Type,
}

impl Parse for FactoryTypeAlias
{
    #[allow(clippy::match_wildcard_for_single_variants)]
    fn parse(input: ParseStream) -> syn::Result<Self>
    {
        let type_alias = match input.parse::<ItemType>() {
            Ok(type_alias) => Ok(type_alias),
            Err(_) => Err(input.error("Expected a type alias")),
        }?;

        let aliased_trait = match &type_alias.ty.as_ref() {
            Type::TraitObject(alias_type) => Ok(alias_type),
            &_ => Err(input.error("Expected the aliased type to be a trait")),
        }?;

        if aliased_trait.bounds.len() != 1 {
            return Err(input.error("Expected the aliased trait to have a single bound."));
        }

        let bound_path = &match aliased_trait.bounds.first().unwrap() {
            TypeParamBound::Trait(trait_bound) => Ok(trait_bound),
            &_ => {
                Err(input.error("Expected the bound of the aliased trait to be a trait"))
            }
        }?
        .path;

        if bound_path.segments.is_empty()
            || bound_path.segments.last().unwrap().ident != "IFactory"
        {
            return Err(input
                .error("Expected the bound of the aliased trait to be 'dyn IFactory'"));
        }

        let angle_bracketed_args = match &bound_path.segments.last().unwrap().arguments {
            syn::PathArguments::AngleBracketed(angle_bracketed_args) => {
                Ok(angle_bracketed_args)
            }
            &_ => {
                Err(input.error("Expected angle bracketed arguments for 'dyn IFactory'"))
            }
        }?;

        let arg_types = match &angle_bracketed_args.args[0] {
            GenericArgument::Type(arg_types_type) => match arg_types_type {
                Type::Tuple(arg_types) => Ok(arg_types),
                &_ => Err(input.error(concat!(
                    "Expected the first angle bracketed argument ",
                    "of 'dyn IFactory' to be a type tuple"
                ))),
            },
            &_ => Err(input.error(concat!(
                "Expected the first angle bracketed argument ",
                "of 'dyn IFactory' to be a type"
            ))),
        }?;

        let return_type = match &angle_bracketed_args.args[1] {
            GenericArgument::Type(arg_type) => Ok(arg_type),
            &_ => Err(input.error(concat!(
                "Expected the second angle bracketed argument ",
                "of 'dyn IFactory' to be a type"
            ))),
        }?;

        Ok(Self {
            type_alias: type_alias.clone(),
            factory_interface: bound_path.clone(),
            arg_types: arg_types.clone(),
            return_type: return_type.clone(),
        })
    }
}
