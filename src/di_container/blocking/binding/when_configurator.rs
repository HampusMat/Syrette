//! When configurator for a binding for types inside of a [`IDIContainer`].
//!
//! [`IDIContainer`]: crate::di_container::blocking::IDIContainer
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::IDIContainer;
use crate::errors::di_container::BindingWhenConfiguratorError;

/// When configurator for a binding for type 'Interface' inside a [`IDIContainer`].
///
/// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
pub struct BindingWhenConfigurator<Interface, DIContainerType>
where
    Interface: 'static + ?Sized,
    DIContainerType: IDIContainer,
{
    di_container: Rc<DIContainerType>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface, DIContainerType> BindingWhenConfigurator<Interface, DIContainerType>
where
    Interface: 'static + ?Sized,
    DIContainerType: IDIContainer,
{
    pub(crate) fn new(di_container: Rc<DIContainerType>) -> Self
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
        let binding = self
            .di_container
            .remove_binding::<Interface>(None)
            .map_or_else(
                || {
                    Err(BindingWhenConfiguratorError::BindingNotFound(type_name::<
                        Interface,
                    >(
                    )))
                },
                Ok,
            )?;

        self.di_container
            .set_binding::<Interface>(Some(name), binding);

        Ok(())
    }
}
