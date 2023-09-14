//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::rc::Rc;

use crate::errors::injectable::InjectableError;
use crate::private::cast::CastFrom;
use crate::ptr::TransientPtr;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

/// Interface for structs that can be injected into or be injected to.
pub trait Injectable<DIContainerT>: CastFrom
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &Rc<DIContainerT>,
        dependency_history: DependencyHistory,
    ) -> Result<TransientPtr<Self>, InjectableError>
    where
        Self: Sized;
}

impl<DIContainerT> Debug for dyn Injectable<DIContainerT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
