//! Smart pointer alias errors.

/// Error type for [`SomePtr`] and [`SomeThreadsafePtr`].
///
/// [`SomePtr`]: crate::ptr::SomePtr
/// [`SomeThreadsafePtr`]: crate::ptr::SomeThreadsafePtr
#[derive(thiserror::Error, Debug)]
pub enum SomePtrError
{
    /// Tried to get as a wrong smart pointer type.
    #[error("Wrong smart pointer type. Expected {expected}, found {found}")]
    WrongPtrType
    {
        /// The expected smart pointer type.
        expected: &'static str,

        /// The found smart pointer type.
        found: &'static str,
    },
}
