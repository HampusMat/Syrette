use core::fmt;
use std::fmt::{Display, Formatter};

use error_stack::Context;

use crate::libs::intertrait::CastFrom;
use crate::DIContainer;

#[derive(Debug)]
pub struct ResolveError;

impl Display for ResolveError
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result
    {
        fmt.write_str("Failed to resolve injectable struct")
    }
}

impl Context for ResolveError {}

pub trait Injectable: CastFrom
{
    fn resolve(
        di_container: &DIContainer,
    ) -> error_stack::Result<Box<Self>, ResolveError>
    where
        Self: Sized;
}
