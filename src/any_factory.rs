//! Interface for any factory to ever exist.

use std::any::Any;
use std::fmt::Debug;

/// Interface for any factory to ever exist.
pub trait AnyFactory: Any + Debug
{
    fn as_any(&self) -> &dyn Any;
}

/// Interface for any threadsafe factory to ever exist.
pub trait AnyThreadsafeFactory: AnyFactory + Send + Sync + Debug {}
