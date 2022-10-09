//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::sync::Arc;

use crate::errors::injectable::InjectableError;
use crate::future::BoxFuture;
use crate::libs::intertrait::CastFromSync;
use crate::ptr::TransientPtr;
use crate::AsyncDIContainer;

/// Interface for structs that can be injected into or be injected to.
pub trait AsyncInjectable: CastFromSync
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve<'di_container, 'fut>(
        di_container: &'di_container Arc<AsyncDIContainer>,
        dependency_history: Vec<&'static str>,
    ) -> BoxFuture<'fut, Result<TransientPtr<Self>, InjectableError>>
    where
        Self: Sized + 'fut,
        'di_container: 'fut;
}

impl Debug for dyn AsyncInjectable
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
