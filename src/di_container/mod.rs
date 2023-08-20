//! Dependency injection container types.

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub mod asynchronous;

pub mod blocking;

/// DI container binding options.
///
/// # Examples
/// ```
/// # use syrette::di_container::BindingOptions;
/// #
/// BindingOptions::new().name("foo");
/// ```
#[derive(Debug, Default, Clone)]
pub struct BindingOptions<'a>
{
    name: Option<&'a str>,
}

impl<'a> BindingOptions<'a>
{
    /// Returns a new `BindingOptions`.
    #[must_use]
    pub fn new() -> Self
    {
        Self { name: None }
    }

    /// Returns `Self` with the specified name set.
    #[must_use]
    pub fn name(mut self, name: &'a str) -> Self
    {
        self.name = Some(name);

        self
    }
}

// Private.
pub(crate) mod binding_storage;
