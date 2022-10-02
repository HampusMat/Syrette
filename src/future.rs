//! Future related utilities.
use std::future::Future;
use std::pin::Pin;

/// A boxed future.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
