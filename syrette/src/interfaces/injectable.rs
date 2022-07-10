use crate::errors::injectable::ResolveError;
use crate::libs::intertrait::CastFrom;
use crate::DIContainer;

pub trait Injectable: CastFrom
{
    fn resolve(
        di_container: &DIContainer,
    ) -> error_stack::Result<Box<Self>, ResolveError>
    where
        Self: Sized;
}
