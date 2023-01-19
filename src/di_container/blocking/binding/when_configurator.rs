//! When configurator for a binding for types inside of a [`IDIContainer`].
//!
//! [`IDIContainer`]: crate::di_container::blocking::IDIContainer
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::dependency_history::IDependencyHistory;
use crate::di_container::blocking::IDIContainer;
use crate::errors::di_container::BindingWhenConfiguratorError;

/// When configurator for a binding for type 'Interface' inside a [`IDIContainer`].
///
/// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
pub struct BindingWhenConfigurator<Interface, DIContainerType, DependencyHistoryType>
where
    Interface: 'static + ?Sized,
    DIContainerType: IDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory,
{
    di_container: Rc<DIContainerType>,

    interface_phantom: PhantomData<Interface>,
    dependency_history_phantom: PhantomData<DependencyHistoryType>,
}

impl<Interface, DIContainerType, DependencyHistoryType>
    BindingWhenConfigurator<Interface, DIContainerType, DependencyHistoryType>
where
    Interface: 'static + ?Sized,
    DIContainerType: IDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory,
{
    pub(crate) fn new(di_container: Rc<DIContainerType>) -> Self
    {
        Self {
            di_container,
            interface_phantom: PhantomData,
            dependency_history_phantom: PhantomData,
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

#[cfg(test)]
mod tests
{
    use mockall::predicate::eq;

    use super::*;
    use crate::provider::blocking::MockIProvider;
    use crate::test_utils::{mocks, subjects};

    #[test]
    fn when_named_works()
    {
        let mut di_container_mock = mocks::blocking_di_container::MockDIContainer::new();

        di_container_mock
            .expect_remove_binding::<dyn subjects::INumber>()
            .with(eq(None))
            .return_once(|_name| Some(Box::new(MockIProvider::new())))
            .once();

        di_container_mock
            .expect_set_binding::<dyn subjects::INumber>()
            .withf(|name, _provider| name == &Some("cool"))
            .return_once(|_name, _provider| ())
            .once();

        let binding_when_configurator = BindingWhenConfigurator::<
            dyn subjects::INumber,
            mocks::blocking_di_container::MockDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(Rc::new(di_container_mock));

        assert!(matches!(
            binding_when_configurator.when_named("cool"),
            Ok(_)
        ));
    }
}
