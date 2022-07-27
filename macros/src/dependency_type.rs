use proc_macro2::Ident;
use syn::{GenericArgument, PathArguments, Type, TypePath};

pub struct DependencyType
{
    pub interface: Type,
    pub ptr: Ident,
}

impl DependencyType
{
    pub fn from_type_path(type_path: &TypePath) -> Option<Self>
    {
        // Assume the type path has a last segment.
        let last_path_segment = type_path.path.segments.last().unwrap();

        let ptr = &last_path_segment.ident;

        match &last_path_segment.arguments {
            PathArguments::AngleBracketed(angle_bracketed_generic_args) => {
                let generic_args = &angle_bracketed_generic_args.args;

                let opt_first_generic_arg = generic_args.first();

                // Assume a first generic argument exists because TransientPtr,
                // SingletonPtr and FactoryPtr requires one
                let first_generic_arg = opt_first_generic_arg.as_ref().unwrap();

                match first_generic_arg {
                    GenericArgument::Type(first_generic_arg_type) => Some(Self {
                        interface: first_generic_arg_type.clone(),
                        ptr: ptr.clone(),
                    }),
                    &_ => None,
                }
            }
            &_ => None,
        }
    }
}
