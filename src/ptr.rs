#![allow(clippy::module_name_repetitions)]

//! Smart pointer type aliases.
use std::rc::Rc;

/// A smart pointer unique to the holder.
pub type TransientPtr<Interface> = Box<Interface>;

/// A smart pointer to a shared resource.
pub type SingletonPtr<Interface> = Rc<Interface>;

/// A smart pointer to a factory.
pub type FactoryPtr<FactoryInterface> = Rc<FactoryInterface>;
