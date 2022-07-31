//! Error types for structs implementing Injectable.

use core::fmt;
use std::fmt::{Display, Formatter};

use error_stack::Context;

/// Error for when a injectable struct fails to be resolved.
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
