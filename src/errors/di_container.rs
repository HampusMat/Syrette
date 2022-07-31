//! Error types for the DI container.

use std::fmt;
use std::fmt::{Display, Formatter};

use error_stack::Context;

/// Error for when the DI container fails to do something.
#[derive(Debug)]
pub struct DIContainerError;

impl Display for DIContainerError
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result
    {
        fmt.write_str("A DI container error has occurred")
    }
}

impl Context for DIContainerError {}

/// Error for when the binding builder fails to do something.
#[derive(Debug)]
pub struct BindingBuilderError;

impl Display for BindingBuilderError
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result
    {
        fmt.write_str("A binding builder error has occurred")
    }
}

impl Context for BindingBuilderError {}
