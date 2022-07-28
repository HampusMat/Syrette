use std::fmt;
use std::fmt::{Display, Formatter};

use error_stack::Context;

#[derive(Debug)]
pub struct CastError;

impl Display for CastError
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result
    {
        fmt.write_str("Failed to cast between traits")
    }
}

impl Context for CastError {}
