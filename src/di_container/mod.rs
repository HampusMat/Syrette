//! Dependency injection container types.

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub mod asynchronous;

pub mod blocking;

// Private.
pub(crate) mod binding_storage;
