use crate::errors::injectable::ResolveError;
use crate::libs::intertrait::CastFrom;
use crate::ptr::InterfacePtr;
use crate::DIContainer;

pub trait Injectable: CastFrom
{
    fn resolve(
        di_container: &DIContainer,
    ) -> error_stack::Result<InterfacePtr<Self>, ResolveError>
    where
        Self: Sized;
}
