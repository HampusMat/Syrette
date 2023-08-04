use std::marker::Tuple;

use crate::private::cast::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for a factory.
pub trait IFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + CastFrom
where
    Args: Tuple,
    ReturnInterface: 'static + ?Sized,
{
}

/// Interface for a threadsafe factory.
#[cfg(feature = "async")]
pub trait IThreadsafeFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + crate::private::cast::CastFromArc
where
    Args: Tuple,
    ReturnInterface: 'static + ?Sized,
{
}
