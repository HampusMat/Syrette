use crate::errors::injectable::ResolveError;
use crate::libs::intertrait::CastFrom;
use crate::ptr::InterfacePtr;
use crate::DIContainer;

pub trait Injectable: CastFrom
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &DIContainer,
    ) -> error_stack::Result<InterfacePtr<Self>, ResolveError>
    where
        Self: Sized;
}
