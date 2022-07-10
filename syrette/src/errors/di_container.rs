use std::fmt;
use std::fmt::{Display, Formatter};

use error_stack::Context;

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
