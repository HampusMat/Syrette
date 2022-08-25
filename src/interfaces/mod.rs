//! Various useful interfaces.

pub mod injectable;

#[cfg(feature = "factory")]
#[doc(hidden)]
pub mod any_factory;

#[cfg(feature = "factory")]
pub mod factory;
