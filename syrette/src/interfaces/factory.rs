#![allow(clippy::module_name_repetitions)]
use crate::libs::intertrait::CastFrom;
use crate::ptr::InterfacePtr;

pub trait IFactory<Args, ReturnInterface>:
    Fn<Args, Output = InterfacePtr<ReturnInterface>> + CastFrom
where
    ReturnInterface: 'static + ?Sized,
{
}
