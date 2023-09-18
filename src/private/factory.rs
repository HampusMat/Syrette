#[cfg(feature = "async")]
use std::sync::Arc;

use crate::private::cast::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for a factory.
pub trait IFactory<ReturnInterface, DIContainerT>: CastFrom
where
    ReturnInterface: 'static + ?Sized,
{
    fn call(&self, di_container: &DIContainerT) -> TransientPtr<ReturnInterface>;
}

/// Interface for a threadsafe factory.
#[cfg(feature = "async")]
pub trait IThreadsafeFactory<ReturnInterface, DIContainerT>:
    Fn<(Arc<DIContainerT>,), Output = TransientPtr<ReturnInterface>>
    + crate::private::cast::CastFromArc
where
    ReturnInterface: 'static + ?Sized,
{
}
