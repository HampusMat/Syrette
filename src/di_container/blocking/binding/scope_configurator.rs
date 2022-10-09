//! Scope configurator for a binding for types inside of a [`DIContainer`].
use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::binding::when_configurator::BindingWhenConfigurator;
use crate::di_container::blocking::DIContainer;
use crate::errors::di_container::BindingScopeConfiguratorError;
use crate::interfaces::injectable::Injectable;
use crate::provider::blocking::{SingletonProvider, TransientTypeProvider};
use crate::ptr::SingletonPtr;

/// Scope configurator for a binding for type 'Interface' inside a [`DIContainer`].
pub struct BindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized,
    Implementation: Injectable,
{
    di_container: Rc<DIContainer>,
    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<Interface, Implementation> BindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized,
    Implementation: Injectable,
{
    pub(crate) fn new(di_container: Rc<DIContainer>) -> Self
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
    #[allow(clippy::must_use_candidate)]
    pub fn in_transient_scope(&self) -> BindingWhenConfigurator<Interface>
    {
        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        bindings_mut.set::<Interface>(
            None,
            Box::new(TransientTypeProvider::<Implementation>::new()),
        );

        BindingWhenConfigurator::new(self.di_container.clone())
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub fn in_singleton_scope(
        &self,
    ) -> Result<BindingWhenConfigurator<Interface>, BindingScopeConfiguratorError>
    {
        let singleton: SingletonPtr<Implementation> = SingletonPtr::from(
            Implementation::resolve(&self.di_container, Vec::new())
                .map_err(BindingScopeConfiguratorError::SingletonResolveFailed)?,
        );

        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        bindings_mut.set::<Interface>(None, Box::new(SingletonProvider::new(singleton)));

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
    }
}
