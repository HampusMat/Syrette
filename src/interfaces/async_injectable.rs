//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::sync::Arc;

use crate::dependency_history::IDependencyHistory;
use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::future::BoxFuture;
use crate::private::cast::CastFromSync;
use crate::ptr::TransientPtr;

/// Interface for structs that can be injected into or be injected to.
pub trait AsyncInjectable<DIContainerType, DependencyHistoryType>: CastFromSync
where
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve<'di_container, 'fut>(
        di_container: &'di_container Arc<DIContainerType>,
        dependency_history: DependencyHistoryType,
    ) -> BoxFuture<'fut, Result<TransientPtr<Self>, InjectableError>>
    where
        Self: Sized + 'fut,
        'di_container: 'fut;
}

impl<DIContainerType, DependencyHistoryType> Debug
    for dyn AsyncInjectable<DIContainerType, DependencyHistoryType>
where
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
