//! Interface for any factory to ever exist.

use std::fmt::Debug;

use crate::private::cast::{CastFrom, CastFromArc};

/// Interface for any factory to ever exist.
pub trait AnyFactory: CastFrom + Debug {}

/// Interface for any threadsafe factory to ever exist.
pub trait AnyThreadsafeFactory: CastFromArc + Debug {}
