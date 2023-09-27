use quote::quote;
use syn::{Generics, Type};

#[cfg(not(tarpaulin_include))]
pub fn expand_dummy_blocking_impl(
    generics: &Generics,
    self_type: &Type,
) -> proc_macro2::TokenStream
{
    quote! {
        impl #generics syrette::interfaces::injectable::Injectable<
            syrette::di_container::blocking::DIContainer,
        > for #self_type
        {
            fn resolve(
                _: &syrette::di_container::blocking::DIContainer,
                _: syrette::dependency_history::DependencyHistory
            ) -> Result<
                syrette::ptr::TransientPtr<Self>,
                syrette::errors::injectable::InjectableError>
            {
                unimplemented!();
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(feature = "async")]
pub fn expand_dummy_async_impl(
    generics: &Generics,
    self_type: &Type,
) -> proc_macro2::TokenStream
{
    quote! {
        impl #generics syrette::interfaces::async_injectable::AsyncInjectable<
            syrette::di_container::asynchronous::AsyncDIContainer,
        > for #self_type
        {
            fn resolve<'di_container, 'fut>(
                _: &'di_container syrette::di_container::asynchronous::AsyncDIContainer,
                _: syrette::dependency_history::DependencyHistory
            ) -> syrette::future::BoxFuture<
                'fut,
                Result<
                    syrette::ptr::TransientPtr<Self>,
                    syrette::errors::injectable::InjectableError
                >
            >
            where
                Self: Sized + 'fut,
                'di_container: 'fut
            {
                unimplemented!();
            }
        }
    }
}
