//! Scope configurator for a binding for types inside of a [`IAsyncDIContainer`].
//!
//! [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
use std::marker::PhantomData;
use std::sync::Arc;

use crate::dependency_history::IDependencyHistory;
use crate::di_container::asynchronous::binding::when_configurator::AsyncBindingWhenConfigurator;
use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::async_di_container::AsyncBindingScopeConfiguratorError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::provider::r#async::{AsyncSingletonProvider, AsyncTransientTypeProvider};
use crate::ptr::ThreadsafeSingletonPtr;

/// Scope configurator for a binding for type 'Interface' inside a [`IAsyncDIContainer`].
///
/// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
pub struct AsyncBindingScopeConfigurator<
    Interface,
    Implementation,
    DIContainerType,
    DependencyHistoryType,
> where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    di_container: Arc<DIContainerType>,
    dependency_history_factory: fn() -> DependencyHistoryType,

    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<Interface, Implementation, DIContainerType, DependencyHistoryType>
    AsyncBindingScopeConfigurator<
        Interface,
        Implementation,
        DIContainerType,
        DependencyHistoryType,
    >
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync + 'static,
{
    pub(crate) fn new(
        di_container: Arc<DIContainerType>,
        dependency_history_factory: fn() -> DependencyHistoryType,
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
    pub async fn in_transient_scope(
        self,
    ) -> AsyncBindingWhenConfigurator<Interface, DIContainerType, DependencyHistoryType>
    {
        self.set_in_transient_scope().await;

        AsyncBindingWhenConfigurator::new(self.di_container.clone())
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub async fn in_singleton_scope(
        self,
    ) -> Result<
        AsyncBindingWhenConfigurator<Interface, DIContainerType, DependencyHistoryType>,
        AsyncBindingScopeConfiguratorError,
    >
    {
        let singleton: ThreadsafeSingletonPtr<Implementation> =
            ThreadsafeSingletonPtr::from(
                Implementation::resolve(
                    &self.di_container,
                    (self.dependency_history_factory)(),
                )
                .await
                .map_err(AsyncBindingScopeConfiguratorError::SingletonResolveFailed)?,
            );

        self.di_container
            .set_binding::<Interface>(
                None,
                Box::new(AsyncSingletonProvider::new(singleton)),
            )
            .await;

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }

    pub(crate) async fn set_in_transient_scope(&self)
    {
        self.di_container
            .set_binding::<Interface>(
                None,
                Box::new(AsyncTransientTypeProvider::<
                    Implementation,
                    DIContainerType,
                    DependencyHistoryType,
                >::new()),
            )
            .await;
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::test_utils::{mocks, subjects_async};

    #[tokio::test]
    async fn in_transient_scope_works()
    {
        let mut di_container_mock =
            mocks::async_di_container::MockAsyncDIContainer::new();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_scope_configurator = AsyncBindingScopeConfigurator::<
            dyn subjects_async::IUserManager,
            subjects_async::UserManager,
            mocks::async_di_container::MockAsyncDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(
            Arc::new(di_container_mock),
            mocks::MockDependencyHistory::new,
        );

        binding_scope_configurator.in_transient_scope().await;
    }

    #[tokio::test]
    async fn in_singleton_scope_works()
    {
        let mut di_container_mock =
            mocks::async_di_container::MockAsyncDIContainer::new();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_scope_configurator = AsyncBindingScopeConfigurator::<
            dyn subjects_async::IUserManager,
            subjects_async::UserManager,
            mocks::async_di_container::MockAsyncDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(
            Arc::new(di_container_mock),
            mocks::MockDependencyHistory::new,
        );

        assert!(matches!(
            binding_scope_configurator.in_singleton_scope().await,
            Ok(_)
        ));
    }
}
