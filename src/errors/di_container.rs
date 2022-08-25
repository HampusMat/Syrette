//! Error types for [`DIContainer`] and it's related structs.
//!
//! [`DIContainer`]: crate::di_container::DIContainer

use crate::errors::injectable::InjectableError;

/// Error type for [`DIContainer`].
///
/// [`DIContainer`]: crate::di_container::DIContainer
#[derive(thiserror::Error, Debug)]
pub enum DIContainerError
{
    /// Unable to cast a binding for a interface.
    #[error("Unable to cast binding for interface '{0}'")]
    CastFailed(&'static str),

    /// Failed to resolve a binding for a interface.
    #[error("Failed to resolve binding for interface '{interface}'")]
    BindingResolveFailed
    {
        /// The reason for the problem.
        #[source]
        reason: InjectableError,

        /// The affected bound interface.
        interface: &'static str,
    },

    /// No binding exists for a interface.
    #[error("No binding exists for interface '{0}'")]
    BindingNotFound(&'static str),

    /// The binding for a interface is a factory but the factory feature isn't enabled.
    #[error("The binding for interface '{0}' is a factory but the factory feature isn't enabled")]
    CantHandleFactoryBinding(&'static str),
}

/// Error type for [`BindingBuilder`].
///
/// [`BindingBuilder`]: crate::di_container::BindingBuilder
#[derive(thiserror::Error, Debug)]
pub enum BindingBuilderError
{
    /// A binding already exists for a interface.
    #[error("Binding already exists for interface '{0}'")]
    BindingAlreadyExists(&'static str),
}

/// Error type for [`BindingScopeConfigurator`].
///
/// [`BindingScopeConfigurator`]: crate::di_container::BindingScopeConfigurator
#[derive(thiserror::Error, Debug)]
pub enum BindingScopeConfiguratorError
{
    /// Resolving a singleton failed.
    #[error("Resolving the given singleton failed")]
    SingletonResolveFailed(#[from] InjectableError),
}
