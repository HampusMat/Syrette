//! Interface for any factory to ever exist.

use std::fmt::Debug;

use crate::libs::intertrait::CastFrom;

/// Interface for any factory to ever exist.
pub trait AnyFactory: CastFrom {}

impl Debug for dyn AnyFactory
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
