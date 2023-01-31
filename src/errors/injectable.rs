//! Error types for structs that implement [`Injectable`].
//!
//! [`Injectable`]: crate::interfaces::injectable::Injectable

use crate::dependency_history::DependencyHistory;
use crate::errors::di_container::DIContainerError;
use crate::errors::ptr::SomePtrError;

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

    /// Failed to resolve dependencies.
    #[cfg(feature = "async")]
    #[error("Failed to resolve a dependency of '{affected}'")]
    AsyncResolveFailed
    {
        /// The reason for the problem.
        #[source]
        reason: Box<crate::errors::async_di_container::AsyncDIContainerError>,

        /// The affected injectable type.
        affected: &'static str,
    },
    /// Detected circular dependencies.
    #[error("Detected circular dependencies. {dependency_history}")]
    DetectedCircular
    {
        /// History of dependencies.
        dependency_history: DependencyHistory,
    },

    /// Failed to prepare a dependency.
    #[error("Failed to prepare dependency '{dependency_name}'")]
    PrepareDependencyFailed
    {
        /// Error reason.
        #[source]
        reason: SomePtrError,

        /// The name of the dependency.
        dependency_name: &'static str,
    },
}
