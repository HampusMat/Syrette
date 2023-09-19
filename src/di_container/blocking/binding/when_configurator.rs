//! When configurator for a binding for types inside of a [`DIContainer`].
use std::any::type_name;
use std::marker::PhantomData;

use crate::di_container::BindingOptions;
use crate::errors::di_container::BindingWhenConfiguratorError;
use crate::util::use_double;

use_double!(crate::di_container::blocking::DIContainer);

/// When configurator for a binding for type `Interface` inside a [`DIContainer`].
pub struct BindingWhenConfigurator<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: &'di_container DIContainer,

    interface_phantom: PhantomData<Interface>,
}

impl<'di_container, Interface> BindingWhenConfigurator<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    pub(crate) fn new(di_container: &'di_container DIContainer) -> Self
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
    ///
    /// # Examples
    /// ```
    /// # use syrette::{DIContainer, injectable};
    /// #
    /// # struct Kitten {}
    /// #
    /// # #[injectable]
    /// # impl Kitten
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = DIContainer::new();
    ///
    /// di_container
    ///     .bind::<Kitten>()
    ///     .to::<Kitten>()?
    ///     .in_transient_scope()
    ///     .when_named("Billy")?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
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
            BindingWhenConfigurator::<dyn subjects::INumber>::new(&di_container_mock);

        assert!(binding_when_configurator.when_named("cool").is_ok());
    }
}
