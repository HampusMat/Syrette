use crate::errors::injectable::ResolveError;
use crate::libs::intertrait::CastFrom;
use crate::ptr::TransientPtr;
use crate::DIContainer;

pub trait Injectable: CastFrom
{
    /// Resolves the dependencies of the injectable.
    ///
    /// # Errors
    /// Will return `Err` if resolving the dependencies fails.
    fn resolve(
        di_container: &DIContainer,
    ) -> error_stack::Result<TransientPtr<Self>, ResolveError>
    where
        Self: Sized;
}
