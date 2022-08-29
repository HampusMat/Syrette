//! Interface for any factory to ever exist.

use std::fmt::Debug;

use crate::libs::intertrait::{CastFrom, CastFromSync};

/// Interface for any factory to ever exist.
pub trait AnyFactory: CastFrom {}

impl Debug for dyn AnyFactory
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}

/// Interface for any threadsafe factory to ever exist.
pub trait AnyThreadsafeFactory: CastFromSync {}

impl Debug for dyn AnyThreadsafeFactory
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("{}")
    }
}
