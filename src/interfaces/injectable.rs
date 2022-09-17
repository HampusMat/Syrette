//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::rc::Rc;

use crate::errors::injectable::InjectableError;
use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;
use crate::DIContainer;

/// Interface for structs that can be injected into or be injected to.
pub trait Injectable: CastFrom
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &Rc<DIContainer>,
        dependency_history: Vec<&'static str>,
    ) -> Result<TransientPtr<Self>, InjectableError>
    where
        Self: Sized;
}

impl Debug for dyn Injectable
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
