#![allow(clippy::module_name_repetitions)]

//! Interface for a factory.
//!
//! ---
//!
//! *This module is only available if Syrette is built with the "factory" feature.*
use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for a factory.
///
/// # Examples
/// ```
/// use syrette::interfaces::factory::IFactory;
///
/// type StringFactory = dyn IFactory<(), String>;
/// ```
pub trait IFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + CastFrom
where
    ReturnInterface: 'static + ?Sized,
{
}
