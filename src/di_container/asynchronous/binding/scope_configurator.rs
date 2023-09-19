//! Scope configurator for a binding for types inside of a [`AsyncDIContainer`].
use std::marker::PhantomData;

use crate::di_container::asynchronous::binding::when_configurator::AsyncBindingWhenConfigurator;
use crate::di_container::BindingOptions;
use crate::errors::async_di_container::AsyncBindingScopeConfiguratorError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::provider::r#async::{AsyncSingletonProvider, AsyncTransientTypeProvider};
use crate::ptr::ThreadsafeSingletonPtr;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);
use_double!(crate::di_container::asynchronous::AsyncDIContainer);

/// Scope configurator for a binding for type `Interface` inside a [`AsyncDIContainer`].
pub struct AsyncBindingScopeConfigurator<'di_container, Interface, Implementation>
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable<AsyncDIContainer>,
{
    di_container: &'di_container AsyncDIContainer,
    dependency_history_factory: fn() -> DependencyHistory,

    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<'di_container, Interface, Implementation>
    AsyncBindingScopeConfigurator<'di_container, Interface, Implementation>
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable<AsyncDIContainer>,
{
    pub(crate) fn new(
        di_container: &'di_container AsyncDIContainer,
        dependency_history_factory: fn() -> DependencyHistory,
    ) -> Self
    {
        Self {
            di_container,
            dependency_history_factory,
            interface_phantom: PhantomData,
            implementation_phantom: PhantomData,
        }
    }

    /// Configures the binding to be in a transient scope.
    ///
    /// This is the default.
    ///
    /// # Examples
    /// ```
    /// # use syrette::{AsyncDIContainer, injectable};
    /// #
    /// # struct Authenticator {}
    /// #
    /// # #[injectable(async = true)]
    /// # impl Authenticator
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = AsyncDIContainer::new();
    ///
    /// di_container
    ///     .bind::<Authenticator>()
    ///     .to::<Authenticator>()
    ///     .await?
    ///     .in_transient_scope()
    ///     .await;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn in_transient_scope(
        self,
    ) -> AsyncBindingWhenConfigurator<'di_container, Interface>
    {
        self.set_in_transient_scope().await;

        AsyncBindingWhenConfigurator::new(self.di_container)
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    ///
    /// # Examples
    /// ```
    /// # use std::sync::atomic::{AtomicBool, Ordering};
    /// # use syrette::{AsyncDIContainer, injectable};
    /// #
    /// # struct AudioManager
    /// # {
    /// #     is_sound_playing: AtomicBool
    /// # }
    /// #
    /// # #[injectable(async = true)]
    /// # impl AudioManager
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self { is_sound_playing: AtomicBool::new(false) }
    /// #     }
    /// #
    /// #     fn play_long_sound(&self)
    /// #     {
    /// #         self.is_sound_playing.store(true, Ordering::Relaxed);
    /// #     }
    /// #
    /// #     fn is_sound_playing(&self) -> bool
    /// #     {
    /// #        self.is_sound_playing.load(Ordering::Relaxed)
    /// #     }
    /// #
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = AsyncDIContainer::new();
    ///
    /// di_container
    ///     .bind::<AudioManager>()
    ///     .to::<AudioManager>()
    ///     .await?
    ///     .in_singleton_scope()
    ///     .await;
    ///
    /// {
    ///     let audio_manager = di_container
    ///         .get::<AudioManager>()
    ///         .await?
    ///         .threadsafe_singleton()?;
    ///
    ///     audio_manager.play_long_sound();
    /// }
    ///
    /// let audio_manager = di_container
    ///     .get::<AudioManager>()
    ///     .await?
    ///     .threadsafe_singleton()?;
    ///
    /// assert!(audio_manager.is_sound_playing());
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn in_singleton_scope(
        self,
    ) -> Result<
        AsyncBindingWhenConfigurator<'di_container, Interface>,
        AsyncBindingScopeConfiguratorError,
    >
    {
        let singleton: ThreadsafeSingletonPtr<Implementation> =
            ThreadsafeSingletonPtr::from(
                Implementation::resolve(
                    self.di_container,
                    (self.dependency_history_factory)(),
                )
                .await
                .map_err(AsyncBindingScopeConfiguratorError::SingletonResolveFailed)?,
            );

        self.di_container
            .set_binding::<Interface>(
                BindingOptions::new(),
                Box::new(AsyncSingletonProvider::new(singleton)),
            )
            .await;

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }

    pub(crate) async fn set_in_transient_scope(&self)
    {
        self.di_container
            .set_binding::<Interface>(
                BindingOptions::new(),
                Box::new(
                    AsyncTransientTypeProvider::<Implementation, AsyncDIContainer>::new(),
                ),
            )
            .await;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::di_container::asynchronous::MockAsyncDIContainer;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn in_transient_scope_works()
    {
        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_scope_configurator =
            AsyncBindingScopeConfigurator::<
                dyn subjects_async::IUserManager,
                subjects_async::UserManager,
            >::new(&di_container_mock, MockDependencyHistory::new);

        binding_scope_configurator.in_transient_scope().await;
    }

    #[tokio::test]
    async fn in_singleton_scope_works()
    {
        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_scope_configurator =
            AsyncBindingScopeConfigurator::<
                dyn subjects_async::IUserManager,
                subjects_async::UserManager,
            >::new(&di_container_mock, MockDependencyHistory::new);

        assert!(binding_scope_configurator
            .in_singleton_scope()
            .await
            .is_ok());
    }
}
