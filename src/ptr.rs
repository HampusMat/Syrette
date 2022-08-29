#![allow(clippy::module_name_repetitions)]

//! Smart pointer type aliases.
use std::rc::Rc;
use std::sync::Arc;

use paste::paste;

use crate::errors::ptr::{SomePtrError, SomeThreadsafePtrError};

/// A smart pointer for a interface in the transient scope.
pub type TransientPtr<Interface> = Box<Interface>;

/// A smart pointer to a interface in the singleton scope.
pub type SingletonPtr<Interface> = Rc<Interface>;

/// A threadsafe smart pointer to a interface in the singleton scope.
pub type ThreadsafeSingletonPtr<Interface> = Arc<Interface>;

/// A smart pointer to a factory.
#[cfg(feature = "factory")]
pub type FactoryPtr<FactoryInterface> = Rc<FactoryInterface>;

/// A threadsafe smart pointer to a factory.
#[cfg(feature = "factory")]
pub type ThreadsafeFactoryPtr<FactoryInterface> = Arc<FactoryInterface>;

macro_rules! create_as_variant_fn {
    ($enum: ident, $variant: ident) => {
        paste! {
            #[doc =
                "Returns as the `" [<$variant>] "` variant.\n"
                "\n"
                "# Errors\n"
                "Will return Err if it's not the `" [<$variant>] "` variant."
            ]
            pub fn [<$variant:snake>](self) -> Result<[<$variant Ptr>]<Interface>, [<$enum Error>]>
            {
                if let $enum::$variant(ptr) = self {
                    return Ok(ptr);
                }


                Err([<$enum Error>]::WrongPtrType {
                    expected: stringify!($variant),
                    found: self.into()
                })
            }
        }
    };
}

/// Some smart pointer.
#[derive(strum_macros::IntoStaticStr)]
pub enum SomePtr<Interface>
where
    Interface: 'static + ?Sized,
{
    /// A smart pointer to a interface in the transient scope.
    Transient(TransientPtr<Interface>),

    /// A smart pointer to a interface in the singleton scope.
    Singleton(SingletonPtr<Interface>),

    /// A smart pointer to a factory.
    #[cfg(feature = "factory")]
    Factory(FactoryPtr<Interface>),
}

impl<Interface> SomePtr<Interface>
where
    Interface: 'static + ?Sized,
{
    create_as_variant_fn!(SomePtr, Transient);

    create_as_variant_fn!(SomePtr, Singleton);

    #[cfg(feature = "factory")]
    create_as_variant_fn!(SomePtr, Factory);
}

/// Some threadsafe smart pointer.
#[derive(strum_macros::IntoStaticStr)]
pub enum SomeThreadsafePtr<Interface>
where
    Interface: 'static + ?Sized,
{
    /// A smart pointer to a interface in the transient scope.
    Transient(TransientPtr<Interface>),

    /// A smart pointer to a interface in the singleton scope.
    ThreadsafeSingleton(ThreadsafeSingletonPtr<Interface>),

    /// A smart pointer to a factory.
    #[cfg(feature = "factory")]
    ThreadsafeFactory(ThreadsafeFactoryPtr<Interface>),
}

impl<Interface> SomeThreadsafePtr<Interface>
where
    Interface: 'static + ?Sized,
{
    create_as_variant_fn!(SomeThreadsafePtr, Transient);

    create_as_variant_fn!(SomeThreadsafePtr, ThreadsafeSingleton);

    #[cfg(feature = "factory")]
    create_as_variant_fn!(SomeThreadsafePtr, ThreadsafeFactory);
}
