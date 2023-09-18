//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;

use crate::errors::injectable::InjectableError;
use crate::future::BoxFuture;
use crate::private::cast::CastFromArc;
use crate::ptr::TransientPtr;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

/// Interface for structs that can be injected into or be injected to.
pub trait AsyncInjectable<DIContainerT>: CastFromArc
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
}

impl<DIContainerT> Debug for dyn AsyncInjectable<DIContainerT>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
