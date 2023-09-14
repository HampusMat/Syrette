//! When configurator for a binding for types inside of a [`DIContainer`].
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::BindingOptions;
use crate::errors::di_container::BindingWhenConfiguratorError;
use crate::util::use_double;

use_double!(crate::di_container::blocking::DIContainer);

/// When configurator for a binding for type `Interface` inside a [`DIContainer`].
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
        self,
        name: &'static str,
    ) -> Result<(), BindingWhenConfiguratorError>
    {
        let binding = self
            .di_container
            .remove_binding::<Interface>(BindingOptions::new())
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
            .set_binding::<Interface>(BindingOptions::new().name(name), binding);

        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use mockall::predicate::eq;

    use super::*;
    use crate::di_container::blocking::MockDIContainer;
    use crate::provider::blocking::MockIProvider;
    use crate::test_utils::subjects;

    #[test]
    fn when_named_works()
    {
        let mut di_container_mock = MockDIContainer::new();

        di_container_mock
            .expect_remove_binding::<dyn subjects::INumber>()
            .with(eq(BindingOptions::new()))
            .return_once(|_name| Some(Box::new(MockIProvider::new())))
            .once();

        di_container_mock
            .expect_set_binding::<dyn subjects::INumber>()
            .withf(|options, _provider| options.name == Some("cool"))
            .return_once(|_name, _provider| ())
            .once();

        let binding_when_configurator =
            BindingWhenConfigurator::<dyn subjects::INumber>::new(Rc::new(
                di_container_mock,
            ));

        assert!(binding_when_configurator.when_named("cool").is_ok());
    }
}
