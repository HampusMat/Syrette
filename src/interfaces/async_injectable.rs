//! Interface for structs that can be injected into or be injected to.
//!
//! *This module is only available if Syrette is built with the "async" feature.*
use std::fmt::Debug;

use async_trait::async_trait;

use crate::async_di_container::AsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::libs::intertrait::CastFromSync;
use crate::ptr::TransientPtr;

/// Interface for structs that can be injected into or be injected to.
#[async_trait]
pub trait AsyncInjectable: CastFromSync
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    async fn resolve(
        di_container: &AsyncDIContainer,
        dependency_history: Vec<&'static str>,
    ) -> Result<TransientPtr<Self>, InjectableError>
    where
        Self: Sized;
}

impl Debug for dyn AsyncInjectable
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
