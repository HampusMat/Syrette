//! Various useful interfaces.

pub mod injectable;

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub mod async_injectable;
