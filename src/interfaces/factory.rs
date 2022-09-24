#![allow(clippy::module_name_repetitions)]
use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for a factory.
pub trait IFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + CastFrom
where
    ReturnInterface: 'static + ?Sized,
{
}
