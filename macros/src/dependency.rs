use std::error::Error;

use proc_macro2::Ident;
use syn::{parse2, FnArg, GenericArgument, LitStr, PathArguments, Type};

use crate::named_attr_input::NamedAttrInput;
use crate::util::syn_path::syn_path_to_string;

pub struct Dependency
{
    pub interface: Type,
    pub ptr: Ident,
    pub name: Option<LitStr>,
}

impl Dependency
{
    pub fn build(new_method_arg: &FnArg) -> Result<Self, Box<dyn Error>>
    {
        let typed_new_method_arg = match new_method_arg {
            FnArg::Typed(typed_arg) => Ok(typed_arg),
            FnArg::Receiver(_) => Err("Unexpected self argument in 'new' method"),
        }?;

        let ptr_type_path = match typed_new_method_arg.ty.as_ref() {
            Type::Path(arg_type_path) => Ok(arg_type_path),
            Type::Reference(ref_type_path) => match ref_type_path.elem.as_ref() {
                Type::Path(arg_type_path) => Ok(arg_type_path),
                &_ => Err("Unexpected reference to non-path type"),
            },
            &_ => Err("Expected a path or a reference type"),
        }?;

        let ptr_path_segment = ptr_type_path.path.segments.last().map_or_else(
            || Err("Expected pointer type path to have a last segment"),
            Ok,
        )?;

        let ptr = ptr_path_segment.ident.clone();

        let ptr_path_generic_args = &match &ptr_path_segment.arguments {
            PathArguments::AngleBracketed(generic_args) => Ok(generic_args),
            &_ => Err("Expected pointer type to have a generic type argument"),
        }?
        .args;

        let interface = if let Some(GenericArgument::Type(interface)) =
            ptr_path_generic_args.first()
        {
            Ok(interface.clone())
        } else {
            Err("Expected pointer type to have a generic type argument")
        }?;

        let arg_attrs = &typed_new_method_arg.attrs;

        let opt_named_attr = arg_attrs.iter().find(|attr| {
            attr.path.get_ident().map_or_else(
                || false,
                |attr_ident| attr_ident.to_string().as_str() == "named",
            ) || syn_path_to_string(&attr.path) == "syrette::named"
        });

        let opt_named_attr_tokens = opt_named_attr.map(|attr| &attr.tokens);

        let opt_named_attr_input =
            if let Some(named_attr_tokens) = opt_named_attr_tokens {
                Some(parse2::<NamedAttrInput>(named_attr_tokens.clone()).map_err(
                    |err| format!("Invalid input for 'named' attribute. {}", err),
                )?)
            } else {
                None
            };

        Ok(Self {
            interface,
            ptr,
            name: opt_named_attr_input.map(|named_attr_input| named_attr_input.name),
        })
    }
}
