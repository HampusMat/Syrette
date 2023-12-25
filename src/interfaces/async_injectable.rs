//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::future::ready;
use std::rc::Rc;
use std::sync::Arc;

use crate::errors::injectable::InjectableError;
use crate::future::BoxFuture;
use crate::ptr::TransientPtr;
use crate::ptr_buffer::PtrBuffer;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

/// Interface for structs that can be injected into or be injected to.
pub trait AsyncInjectable<DIContainerT>: 'static + Send + Sync
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve<'di_container, 'fut>(
        di_container: &'di_container DIContainerT,
        dependency_history: DependencyHistory,
    ) -> BoxFuture<'fut, Result<TransientPtr<Self>, InjectableError>>
    where
        Self: Sized + 'fut,
        'di_container: 'fut;

    /// A.
    fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer;

    /// A.
    fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer;

    /// A.
    fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer;
}

impl<DIContainerT> Debug for dyn AsyncInjectable<DIContainerT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}

impl<T, DIContainerT> AsyncInjectable<DIContainerT> for T
where
    T: Default + 'static + Send + Sync,
{
    fn resolve<'di_container, 'fut>(
        _: &'di_container DIContainerT,
        _: DependencyHistory,
    ) -> BoxFuture<'fut, Result<TransientPtr<Self>, InjectableError>>
    where
        Self: Sized + 'fut,
        'di_container: 'fut,
    {
        Box::pin(ready(Ok(TransientPtr::new(Self::default()))))
    }

    fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer
    {
        PtrBuffer::new_from(self)
    }

    fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer
    {
        PtrBuffer::new_from(self)
    }

    fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer
    {
        PtrBuffer::new_from(self)
    }
}
