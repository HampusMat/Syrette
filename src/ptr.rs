#![allow(clippy::module_name_repetitions)]

//! Smart pointer type aliases.
use std::rc::Rc;

use paste::paste;

use crate::errors::ptr::SomePtrError;

/// A smart pointer for a interface in the transient scope.
pub type TransientPtr<Interface> = Box<Interface>;

/// A smart pointer to a interface in the singleton scope.
pub type SingletonPtr<Interface> = Rc<Interface>;

/// A smart pointer to a factory.
#[cfg(feature = "factory")]
pub type FactoryPtr<FactoryInterface> = Rc<FactoryInterface>;

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

macro_rules! create_as_variant_fn {
    ($variant: ident) => {
        paste! {
            #[doc =
                "Returns as " [<$variant:lower>] ".\n"
                "\n"
                "# Errors\n"
                "Will return Err if it's not a " [<$variant:lower>] "."
            ]
            pub fn [<$variant:lower>](self) -> Result<[<$variant Ptr>]<Interface>, SomePtrError>
            {
                if let SomePtr::$variant(ptr) = self {
                    return Ok(ptr);
                }


                Err(SomePtrError::WrongPtrType {
                    expected: stringify!($variant),
                    found: self.into()
                })
            }
        }
    };
}

impl<Interface> SomePtr<Interface>
where
    Interface: 'static + ?Sized,
{
    create_as_variant_fn!(Transient);
    create_as_variant_fn!(Singleton);

    #[cfg(feature = "factory")]
    create_as_variant_fn!(Factory);
}
