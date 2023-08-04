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
                syrette::private::castable_factory::threadsafe::ThreadsafeCastableFactory<
                    (std::sync::Arc<syrette::AsyncDIContainer>,),
                    #factory_interface
                > -> syrette::private::factory::IThreadsafeFactory<
                    (std::sync::Arc<syrette::AsyncDIContainer>,),
                    #factory_interface
                >,
                threadsafe_sharable = true
            );

            syrette::declare_interface!(
                syrette::private::castable_factory::threadsafe::ThreadsafeCastableFactory<
                    (std::sync::Arc<syrette::AsyncDIContainer>,),
                    #factory_interface
                > -> syrette::private::any_factory::AnyThreadsafeFactory,
                threadsafe_sharable = true
            );
        }
    } else {
        quote! {
            syrette::declare_interface!(
                syrette::private::castable_factory::blocking::CastableFactory<
                    (std::rc::Rc<syrette::DIContainer>,),
                    #factory_interface
                > -> syrette::private::factory::IFactory<
                    (std::rc::Rc<syrette::DIContainer>,),
                    #factory_interface
                >
            );

            syrette::declare_interface!(
                syrette::private::castable_factory::blocking::CastableFactory<
                    (std::rc::Rc<syrette::DIContainer>,),
                    #factory_interface
                > -> syrette::private::any_factory::AnyFactory
            );
        }
    }
}
