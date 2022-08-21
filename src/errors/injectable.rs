#![allow(clippy::module_name_repetitions)]
//! Error types for structs that implement [`Injectable`].
//!
//! [`Injectable`]: crate::interfaces::injectable::Injectable

use super::di_container::DIContainerError;

/// Error type for structs that implement [`Injectable`].
///
/// [`Injectable`]: crate::interfaces::injectable::Injectable
#[derive(thiserror::Error, Debug)]
pub enum InjectableError
{
    /// Failed to resolve dependencies.
    #[error("Failed to resolve a dependency of '{affected}'")]
    ResolveFailed
    {
        /// The reason for the problem.
        #[source]
        reason: Box<DIContainerError>,

        /// The affected injectable type.
        affected: &'static str,
    },

    /// Detected circular dependencies.
    #[error("Detected circular dependencies. {dependency_trace}")]
    DetectedCircular
    {
        /// A visual trace of dependencies.
        dependency_trace: String,
    },
}
