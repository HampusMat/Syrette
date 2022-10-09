//! Scope configurator for a binding for types inside of a [`AsyncDIContainer`].
use std::marker::PhantomData;
use std::sync::Arc;

use crate::di_container::asynchronous::binding::when_configurator::AsyncBindingWhenConfigurator;
use crate::errors::async_di_container::AsyncBindingScopeConfiguratorError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::provider::r#async::{AsyncSingletonProvider, AsyncTransientTypeProvider};
use crate::ptr::ThreadsafeSingletonPtr;
use crate::AsyncDIContainer;

/// Scope configurator for a binding for type 'Interface' inside a [`AsyncDIContainer`].
pub struct AsyncBindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable,
{
    di_container: Arc<AsyncDIContainer>,
    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<Interface, Implementation> AsyncBindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable,
{
    pub(crate) fn new(di_container: Arc<AsyncDIContainer>) -> Self
    {
        Self {
            di_container,
            interface_phantom: PhantomData,
            implementation_phantom: PhantomData,
        }
    }

    /// Configures the binding to be in a transient scope.
    ///
    /// This is the default.
    pub async fn in_transient_scope(&self) -> AsyncBindingWhenConfigurator<Interface>
    {
        let mut bindings_lock = self.di_container.bindings.lock().await;

        bindings_lock.set::<Interface>(
            None,
            Box::new(AsyncTransientTypeProvider::<Implementation>::new()),
        );

        AsyncBindingWhenConfigurator::new(self.di_container.clone())
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub async fn in_singleton_scope(
        &self,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingScopeConfiguratorError>
    {
        let singleton: ThreadsafeSingletonPtr<Implementation> =
            ThreadsafeSingletonPtr::from(
                Implementation::resolve(&self.di_container, Vec::new())
                    .await
                    .map_err(
                        AsyncBindingScopeConfiguratorError::SingletonResolveFailed,
                    )?,
            );

        let mut bindings_lock = self.di_container.bindings.lock().await;

        bindings_lock
            .set::<Interface>(None, Box::new(AsyncSingletonProvider::new(singleton)));

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }
}
