//! Interface for any factory to ever exist.

use std::fmt::Debug;

use crate::private::cast::{CastFrom, CastFromSync};

/// Interface for any factory to ever exist.
pub trait AnyFactory: CastFrom + Debug {}

/// Interface for any threadsafe factory to ever exist.
pub trait AnyThreadsafeFactory: CastFromSync + Debug {}
