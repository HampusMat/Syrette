//! Smart pointer alias errors.

/// Error type for [`SomePtr`].
///
/// [`SomePtr`]: crate::ptr::SomePtr
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

/// Error type for [`SomeThreadsafePtr`].
///
/// [`SomeThreadsafePtr`]: crate::ptr::SomeThreadsafePtr
#[derive(thiserror::Error, Debug)]
pub enum SomeThreadsafePtrError
{
    /// Tried to get as a wrong threadsafe smart pointer type.
    #[error("Wrong threadsafe smart pointer type. Expected {expected}, found {found}")]
    WrongPtrType
    {
        /// The expected smart pointer type.
        expected: &'static str,

        /// The found smart pointer type.
        found: &'static str,
    },
}
