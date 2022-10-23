//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::sync::Arc;

use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::future::BoxFuture;
use crate::libs::intertrait::CastFromSync;
use crate::ptr::TransientPtr;

/// Interface for structs that can be injected into or be injected to.
pub trait AsyncInjectable<DIContainerType>: CastFromSync
where
    DIContainerType: IAsyncDIContainer,
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve<'di_container, 'fut>(
        di_container: &'di_container Arc<DIContainerType>,
        dependency_history: Vec<&'static str>,
    ) -> BoxFuture<'fut, Result<TransientPtr<Self>, InjectableError>>
    where
        Self: Sized + 'fut,
        'di_container: 'fut;
}

impl<DIContainerType> Debug for dyn AsyncInjectable<DIContainerType>
where
    DIContainerType: IAsyncDIContainer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
