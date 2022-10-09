//! When configurator for a binding for types inside of a [`AsyncDIContainer`].
use std::any::type_name;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::errors::async_di_container::AsyncBindingWhenConfiguratorError;
use crate::AsyncDIContainer;

/// When configurator for a binding for type 'Interface' inside a [`AsyncDIContainer`].
pub struct AsyncBindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    di_container: Arc<AsyncDIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> AsyncBindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    pub(crate) fn new(di_container: Arc<AsyncDIContainer>) -> Self
    {
        Self {
            di_container,
            interface_phantom: PhantomData,
        }
    }

    /// Configures the binding to have a name.
    ///
    /// # Errors
    /// Will return Err if no binding for the interface already exists.
    pub async fn when_named(
        &self,
        name: &'static str,
    ) -> Result<(), AsyncBindingWhenConfiguratorError>
    {
        let mut bindings_lock = self.di_container.bindings.lock().await;

        let binding = bindings_lock.remove::<Interface>(None).map_or_else(
            || {
                Err(AsyncBindingWhenConfiguratorError::BindingNotFound(
                    type_name::<Interface>(),
                ))
            },
            Ok,
        )?;

        bindings_lock.set::<Interface>(Some(name), binding);

        Ok(())
    }
}
