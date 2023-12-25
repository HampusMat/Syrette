//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::Arc;

use crate::errors::injectable::InjectableError;
use crate::ptr::TransientPtr;
use crate::ptr_buffer::{PtrBuffer, SmartPtr};
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

/// Interface for structs that can be injected into or be injected to.
pub trait Injectable<DIContainerT>: 'static
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &DIContainerT,
        dependency_history: DependencyHistory,
    ) -> Result<TransientPtr<Self>, InjectableError>
    where
        Self: Sized;

    /// A.
    fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer;

    /// A.
    fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer;

    /// A.
    fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer;
}

impl<DIContainerT> Debug for dyn Injectable<DIContainerT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}

impl<T, DIContainerT> Injectable<DIContainerT> for T
where
    T: Default + 'static,
{
    fn resolve(
        _: &DIContainerT,
        _: DependencyHistory,
    ) -> Result<TransientPtr<Self>, InjectableError>
    {
        Ok(TransientPtr::new(Self::default()))
    }

    fn into_ptr_buffer_box(self: Box<Self>) -> PtrBuffer
    {
        PtrBuffer::new_from(SmartPtr::Box(self))
    }

    fn into_ptr_buffer_rc(self: Rc<Self>) -> PtrBuffer
    {
        PtrBuffer::new_from(SmartPtr::Rc(self))
    }

    fn into_ptr_buffer_arc(self: Arc<Self>) -> PtrBuffer
    {
        PtrBuffer::new_from(SmartPtr::Arc(self))
    }
}
