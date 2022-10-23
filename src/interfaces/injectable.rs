//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::rc::Rc;

use crate::di_container::blocking::IDIContainer;
use crate::errors::injectable::InjectableError;
use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for structs that can be injected into or be injected to.
pub trait Injectable<DIContainerType>: CastFrom
where
    DIContainerType: IDIContainer,
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &Rc<DIContainerType>,
        dependency_history: Vec<&'static str>,
    ) -> Result<TransientPtr<Self>, InjectableError>
    where
        Self: Sized;
}

impl<DIContainerType> Debug for dyn Injectable<DIContainerType>
where
    DIContainerType: IDIContainer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
