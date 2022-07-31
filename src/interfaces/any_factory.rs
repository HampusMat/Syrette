//! Interface for any factory to ever exist.

use crate::libs::intertrait::CastFrom;

/// Interface for any factory to ever exist.
pub trait AnyFactory: CastFrom {}
