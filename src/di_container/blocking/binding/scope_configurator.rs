//! Scope configurator for a binding for types inside of a [`IDIContainer`].
//!
//! [`IDIContainer`]: crate::di_container::blocking::IDIContainer
use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::binding::when_configurator::BindingWhenConfigurator;
use crate::di_container::blocking::IDIContainer;
use crate::errors::di_container::BindingScopeConfiguratorError;
use crate::interfaces::injectable::Injectable;
use crate::provider::blocking::{SingletonProvider, TransientTypeProvider};
use crate::ptr::SingletonPtr;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

/// Scope configurator for a binding for type `Interface` inside a [`IDIContainer`].
///
/// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
pub struct BindingScopeConfigurator<Interface, Implementation, DIContainerType>
where
    Interface: 'static + ?Sized,
    Implementation: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    di_container: Rc<DIContainerType>,
    dependency_history_factory: fn() -> DependencyHistory,

    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<Interface, Implementation, DIContainerType>
    BindingScopeConfigurator<Interface, Implementation, DIContainerType>
where
    Interface: 'static + ?Sized,
    Implementation: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    pub(crate) fn new(
        di_container: Rc<DIContainerType>,
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
    #[allow(clippy::must_use_candidate)]
    pub fn in_transient_scope(self)
        -> BindingWhenConfigurator<Interface, DIContainerType>
    {
        self.set_in_transient_scope();

        BindingWhenConfigurator::new(self.di_container)
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub fn in_singleton_scope(
        self,
    ) -> Result<
        BindingWhenConfigurator<Interface, DIContainerType>,
        BindingScopeConfiguratorError,
    >
    {
        let singleton: SingletonPtr<Implementation> = SingletonPtr::from(
            Implementation::resolve(
                &self.di_container,
                (self.dependency_history_factory)(),
            )
            .map_err(BindingScopeConfiguratorError::SingletonResolveFailed)?,
        );

        self.di_container
            .set_binding::<Interface>(None, Box::new(SingletonProvider::new(singleton)));

        Ok(BindingWhenConfigurator::new(self.di_container))
    }

    pub(crate) fn set_in_transient_scope(&self)
    {
        self.di_container.set_binding::<Interface>(
            None,
            Box::new(TransientTypeProvider::<Implementation, DIContainerType>::new()),
        );
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::test_utils::{mocks, subjects};

    #[test]
    fn in_transient_scope_works()
    {
        let mut di_container_mock = mocks::blocking_di_container::MockDIContainer::new();

        di_container_mock
            .expect_set_binding::<dyn subjects::IUserManager>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_scope_configurator =
            BindingScopeConfigurator::<
                dyn subjects::IUserManager,
                subjects::UserManager,
                mocks::blocking_di_container::MockDIContainer,
            >::new(Rc::new(di_container_mock), MockDependencyHistory::new);

        binding_scope_configurator.in_transient_scope();
    }

    #[test]
    fn in_singleton_scope_works()
    {
        let mut di_container_mock = mocks::blocking_di_container::MockDIContainer::new();

        di_container_mock
            .expect_set_binding::<dyn subjects::IUserManager>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_scope_configurator =
            BindingScopeConfigurator::<
                dyn subjects::IUserManager,
                subjects::UserManager,
                mocks::blocking_di_container::MockDIContainer,
            >::new(Rc::new(di_container_mock), MockDependencyHistory::new);

        assert!(binding_scope_configurator.in_singleton_scope().is_ok());
    }
}
