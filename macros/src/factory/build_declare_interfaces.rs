use proc_macro2::TokenStream;
use quote::quote;

use crate::fn_trait::FnTrait;

pub fn build_declare_factory_interfaces(
    factory_interface: &FnTrait,
    is_threadsafe: bool,
) -> TokenStream
{
    if is_threadsafe {
        quote! {
            syrette::declare_interface!(
                syrette::castable_factory::threadsafe::ThreadsafeCastableFactory<
                    (std::sync::Arc<syrette::async_di_container::AsyncDIContainer>,),
                    #factory_interface
                > -> syrette::interfaces::factory::IThreadsafeFactory<
                    (std::sync::Arc<syrette::async_di_container::AsyncDIContainer>,),
                    #factory_interface
                >,
                async = true
            );

            syrette::declare_interface!(
                syrette::castable_factory::threadsafe::ThreadsafeCastableFactory<
                    (std::sync::Arc<syrette::async_di_container::AsyncDIContainer>,),
                    #factory_interface
                > -> syrette::interfaces::any_factory::AnyThreadsafeFactory,
                async = true
            );
        }
    } else {
        quote! {
            syrette::declare_interface!(
                syrette::castable_factory::blocking::CastableFactory<
                    (std::rc::Rc<syrette::di_container::DIContainer>,),
                    #factory_interface
                > -> syrette::interfaces::factory::IFactory<
                    (std::rc::Rc<syrette::di_container::DIContainer>,),
                    #factory_interface
                >
            );

            syrette::declare_interface!(
                syrette::castable_factory::blocking::CastableFactory<
                    (std::rc::Rc<syrette::di_container::DIContainer>,),
                    #factory_interface
                > -> syrette::interfaces::any_factory::AnyFactory
            );
        }
    }
}
