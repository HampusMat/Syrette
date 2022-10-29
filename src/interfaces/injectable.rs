//! Interface for structs that can be injected into or be injected to.
use std::fmt::Debug;
use std::rc::Rc;

use crate::dependency_history::IDependencyHistory;
use crate::di_container::blocking::IDIContainer;
use crate::errors::injectable::InjectableError;
use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;

/// Interface for structs that can be injected into or be injected to.
pub trait Injectable<DIContainerType, DependencyHistoryType>: CastFrom
where
    DIContainerType: IDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory,
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &Rc<DIContainerType>,
        dependency_history: DependencyHistoryType,
    ) -> Result<TransientPtr<Self>, InjectableError>
    where
        Self: Sized;
}

impl<DIContainerType, DependencyHistoryType> Debug
    for dyn Injectable<DIContainerType, DependencyHistoryType>
where
    DIContainerType: IDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
