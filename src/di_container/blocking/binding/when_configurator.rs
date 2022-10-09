//! When configurator for a binding for types inside of a [`DIContainer`].
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::DIContainer;
use crate::errors::di_container::BindingWhenConfiguratorError;

/// When configurator for a binding for type 'Interface' inside a [`DIContainer`].
pub struct BindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: Rc<DIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> BindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized,
{
    pub(crate) fn new(di_container: Rc<DIContainer>) -> Self
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
    pub fn when_named(
        &self,
        name: &'static str,
    ) -> Result<(), BindingWhenConfiguratorError>
    {
        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        let binding = bindings_mut.remove::<Interface>(None).map_or_else(
            || {
                Err(BindingWhenConfiguratorError::BindingNotFound(type_name::<
                    Interface,
                >(
                )))
            },
            Ok,
        )?;

        bindings_mut.set::<Interface>(Some(name), binding);

        Ok(())
    }
}
