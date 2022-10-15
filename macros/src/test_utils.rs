use proc_macro2::{Ident, TokenStream};
use syn::punctuated::Punctuated;
use syn::token::{Colon, Gt, Lt, Paren, RArrow};
use syn::{
    AngleBracketedGenericArguments,
    Attribute,
    FnArg,
    GenericArgument,
    GenericParam,
    Generics,
    Pat,
    PatType,
    Path,
    PathArguments,
    PathSegment,
    Signature,
    Type,
    TypePath,
};

pub fn create_path(segments: &[PathSegment]) -> Path
{
    Path {
        leading_colon: None,
        segments: segments.iter().cloned().collect(),
    }
}

pub fn create_path_segment(ident: Ident, generic_arg_types: &[Type]) -> PathSegment
{
    PathSegment {
        ident,
        arguments: if generic_arg_types.is_empty() {
            PathArguments::None
        } else {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Lt::default(),
                args: generic_arg_types
                    .iter()
                    .map(|generic_arg_type| {
                        GenericArgument::Type(generic_arg_type.clone())
                    })
                    .collect(),
                gt_token: Gt::default(),
            })
        },
    }
}

pub fn create_type(path: Path) -> Type
{
    Type::Path(TypePath { qself: None, path })
}

pub fn create_generics<Params>(params: Params) -> Generics
where
    Params: IntoIterator<Item = GenericParam>,
{
    Generics {
        lt_token: None,
        params: Punctuated::from_iter(params),
        gt_token: None,
        where_clause: None,
    }
}

pub fn create_signature<ArgTypes>(
    ident: Ident,
    arg_types: ArgTypes,
    return_type: Type,
) -> Signature
where
    ArgTypes: IntoIterator<Item = (Type, Vec<Attribute>)>,
{
    Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: syn::token::Fn::default(),
        ident,
        generics: create_generics(vec![]),
        paren_token: Paren::default(),
        inputs: arg_types
            .into_iter()
            .map(|(arg_type, attrs)| {
                FnArg::Typed(PatType {
                    attrs,
                    pat: Box::new(Pat::Verbatim(TokenStream::new())),
                    colon_token: Colon::default(),
                    ty: Box::new(arg_type),
                })
            })
            .collect(),
        variadic: None,
        output: syn::ReturnType::Type(RArrow::default(), Box::new(return_type)),
    }
}
