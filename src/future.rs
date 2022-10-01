//! Future related utilities.
//!
//! *This module is only available if Syrette is built with the "async" feature.*
use std::future::Future;
use std::pin::Pin;

/// A boxed future.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
