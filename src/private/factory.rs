use std::rc::Rc;

use crate::private::cast::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for a factory.
pub trait IFactory<ReturnInterface, DIContainerT>: CastFrom
where
    ReturnInterface: 'static + ?Sized,
{
    fn call(&self, di_container: Rc<DIContainerT>) -> TransientPtr<ReturnInterface>;
}

/// Interface for a threadsafe factory.
#[cfg(feature = "async")]
pub trait IThreadsafeFactory<Args, ReturnInterface>:
    Fn<Args, Output = TransientPtr<ReturnInterface>> + crate::private::cast::CastFromArc
where
    Args: std::marker::Tuple,
    ReturnInterface: 'static + ?Sized,
{
}
