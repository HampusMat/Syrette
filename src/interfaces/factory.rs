use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for a factory.
pub trait IFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + CastFrom
where
    ReturnInterface: 'static + ?Sized,
{
}

/// Interface for a threadsafe factory.
#[cfg(feature = "async")]
pub trait IThreadsafeFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + crate::libs::intertrait::CastFromSync
where
    ReturnInterface: 'static + ?Sized,
{
}
