//! Various useful interfaces.

use feature_macros::feature_specific;

pub mod injectable;

#[cfg(feature = "factory")]
#[doc(hidden)]
pub mod any_factory;

#[cfg(feature = "factory")]
#[doc(hidden)]
pub mod factory;

#[feature_specific("async")]
pub mod async_injectable;
