//! Binding builder for types inside of a [`IDIContainer`].
//!
//! [`IDIContainer`]: crate::di_container::blocking::IDIContainer
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::dependency_history::IDependencyHistory;
use crate::di_container::blocking::binding::scope_configurator::BindingScopeConfigurator;
#[cfg(feature = "factory")]
use crate::di_container::blocking::binding::when_configurator::BindingWhenConfigurator;
use crate::di_container::blocking::IDIContainer;
use crate::errors::di_container::BindingBuilderError;
use crate::interfaces::injectable::Injectable;

/// Binding builder for type `Interface` inside a [`IDIContainer`].
///
/// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
pub struct BindingBuilder<Interface, DIContainerType, DependencyHistoryType>
where
    Interface: 'static + ?Sized,
    DIContainerType: IDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory,
{
    di_container: Rc<DIContainerType>,
    dependency_history_factory: fn() -> DependencyHistoryType,

    interface_phantom: PhantomData<Interface>,
}

impl<Interface, DIContainerType, DependencyHistoryType>
    BindingBuilder<Interface, DIContainerType, DependencyHistoryType>
where
    Interface: 'static + ?Sized,
    DIContainerType: IDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + 'static,
{
    pub(crate) fn new(
        di_container: Rc<DIContainerType>,
        dependency_history_factory: fn() -> DependencyHistoryType,
    ) -> Self
    {
        Self {
            di_container,
            dependency_history_factory,
            interface_phantom: PhantomData,
        }
    }

    /// Creates a binding of type `Interface` to type `Implementation` inside of the
    /// associated [`IDIContainer`].
    ///
    /// The scope of the binding is transient. But that can be changed by using the
    /// returned [`BindingScopeConfigurator`]
    ///
    /// # Errors
    /// Will return Err if the associated [`IDIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{DIContainer, injectable};
    /// # use syrette::di_container::blocking::IDIContainer;
    /// #
    /// # trait Foo {}
    /// #
    /// # struct Bar {}
    /// #
    /// # #[injectable(Foo)]
    /// # impl Bar {
    /// #   fn new() -> Self
    /// #   {
    /// #       Self {}
    /// #   }
    /// # }
    /// #
    /// # impl Foo for Bar {}
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = DIContainer::new();
    /// #
    /// di_container.bind::<dyn Foo>().to::<Bar>();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
    pub fn to<Implementation>(
        &self,
    ) -> Result<
        BindingScopeConfigurator<
            Interface,
            Implementation,
            DIContainerType,
            DependencyHistoryType,
        >,
        BindingBuilderError,
    >
    where
        Implementation: Injectable<DIContainerType, DependencyHistoryType>,
    {
        {
            if self.di_container.has_binding::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let binding_scope_configurator = BindingScopeConfigurator::new(
            self.di_container.clone(),
            self.dependency_history_factory,
        );

        binding_scope_configurator.in_transient_scope();

        Ok(binding_scope_configurator)
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`IDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`IDIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{DIContainer, factory};
    /// # use syrette::ptr::TransientPtr;
    /// # use syrette::di_container::blocking::IDIContainer;
    /// #
    /// # trait ICustomerID {}
    /// # trait ICustomer {}
    /// #
    /// # struct Customer
    /// # {
    /// #   name: String,
    /// #   id: TransientPtr<dyn ICustomerID>
    /// # }
    /// #
    /// # impl Customer {
    /// #   fn new(name: String, id: TransientPtr<dyn ICustomerID>) -> Self
    /// #   {
    /// #       Self { name, id }
    /// #   }
    /// # }
    /// #
    /// # impl ICustomer for Customer {}
    /// #
    /// # #[factory]
    /// # type ICustomerFactory = dyn Fn(String, u32) -> dyn ICustomer;
    /// #
    /// # #[factory]
    /// # type ICustomerIDFactory = dyn Fn(u32) -> dyn ICustomerID;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = DIContainer::new();
    /// #
    /// di_container
    ///     .bind::<ICustomerFactory>()
    ///     .to_factory(&|context| {
    ///         Box::new(move |name, id| {
    ///             let customer_id_factory = context
    ///                 .get::<ICustomerIDFactory>()
    ///                 .unwrap()
    ///                 .factory()
    ///                 .unwrap();
    ///
    ///             let customer_id = customer_id_factory(id);
    ///
    ///             let customer = TransientPtr::new(Customer::new(name, customer_id));
    ///
    ///             customer as TransientPtr<dyn ICustomer>
    ///         })
    ///     });
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_factory<Args, Return, Func>(
        &self,
        factory_func: &'static Func,
    ) -> Result<
        BindingWhenConfigurator<Interface, DIContainerType, DependencyHistoryType>,
        BindingBuilderError,
    >
    where
        Args: std::marker::Tuple + 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = crate::ptr::TransientPtr<Return>>,
        Func: Fn<(std::rc::Rc<DIContainerType>,), Output = Box<Interface>>,
    {
        use crate::private::castable_factory::blocking::CastableFactory;

        if self.di_container.has_binding::<Interface>(None) {
            return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >()));
        }

        let factory_impl = CastableFactory::new(factory_func);

        self.di_container.set_binding::<Interface>(
            None,
            Box::new(crate::provider::blocking::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
                false,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
    }

    /// Creates a binding of type `Interface` to a factory that takes no arguments
    /// inside of the associated [`IDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`IDIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{DIContainer, factory};
    /// # use syrette::ptr::TransientPtr;
    /// # use syrette::di_container::blocking::IDIContainer;
    /// #
    /// # trait IBuffer {}
    /// #
    /// # struct Buffer<const SIZE: usize>
    /// # {
    /// #   buf: [u8; SIZE]
    /// # }
    /// #
    /// # impl<const SIZE: usize> Buffer<SIZE>
    /// # {
    /// #   fn new() -> Self
    /// #   {
    /// #       Self {
    /// #           buf: [0; SIZE]
    /// #       }
    /// #   }
    /// # }
    /// #
    /// # impl<const SIZE: usize> IBuffer for Buffer<SIZE> {}
    /// #
    /// # const BUFFER_SIZE: usize = 12;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = DIContainer::new();
    /// #
    /// di_container.bind::<dyn IBuffer>().to_default_factory(&|_| {
    ///     Box::new(|| {
    ///         let buffer = TransientPtr::new(Buffer::<BUFFER_SIZE>::new());
    ///
    ///         buffer as TransientPtr<dyn IBuffer>
    ///     })
    /// });
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IDIContainer`]: crate::di_container::blocking::IDIContainer
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        BindingWhenConfigurator<Interface, DIContainerType, DependencyHistoryType>,
        BindingBuilderError,
    >
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
            (Rc<DIContainerType>,),
            Output = crate::ptr::TransientPtr<
                dyn Fn<(), Output = crate::ptr::TransientPtr<Return>>,
            >,
        >,
    {
        use crate::private::castable_factory::blocking::CastableFactory;

        if self.di_container.has_binding::<Interface>(None) {
            return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >()));
        }

        let factory_impl = CastableFactory::new(factory_func);

        self.di_container.set_binding::<Interface>(
            None,
            Box::new(crate::provider::blocking::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
                true,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use mockall::predicate::eq;

    use super::*;
    use crate::test_utils::{mocks, subjects};

    #[test]
    fn can_bind_to() -> Result<(), Box<dyn Error>>
    {
        let mut mock_di_container = mocks::blocking_di_container::MockDIContainer::new();

        mock_di_container
            .expect_has_binding::<dyn subjects::INumber>()
            .with(eq(None))
            .return_once(|_name| false)
            .once();

        mock_di_container
            .expect_set_binding::<dyn subjects::INumber>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder = BindingBuilder::<
            dyn subjects::INumber,
            mocks::blocking_di_container::MockDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(
            Rc::new(mock_di_container),
            mocks::MockDependencyHistory::new,
        );

        binding_builder.to::<subjects::Number>()?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory() -> Result<(), Box<dyn Error>>
    {
        use crate as syrette;
        use crate::factory;
        use crate::ptr::TransientPtr;

        #[factory]
        type IUserManagerFactory = dyn Fn(i32, String) -> dyn subjects::IUserManager;

        let mut mock_di_container = mocks::blocking_di_container::MockDIContainer::new();

        mock_di_container
            .expect_has_binding::<IUserManagerFactory>()
            .with(eq(None))
            .return_once(|_name| false)
            .once();

        mock_di_container
            .expect_set_binding::<IUserManagerFactory>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder = BindingBuilder::<
            IUserManagerFactory,
            mocks::blocking_di_container::MockDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(
            Rc::new(mock_di_container),
            mocks::MockDependencyHistory::new,
        );

        binding_builder.to_factory(&|_| {
            Box::new(move |_num, _text| {
                let user_manager: TransientPtr<dyn subjects::IUserManager> =
                    TransientPtr::new(subjects::UserManager::new());

                user_manager
            })
        })?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_default_factory() -> Result<(), Box<dyn Error>>
    {
        use syrette_macros::declare_default_factory;

        use crate as syrette;
        use crate::ptr::TransientPtr;

        declare_default_factory!(dyn subjects::IUserManager);

        let mut mock_di_container = mocks::blocking_di_container::MockDIContainer::new();

        mock_di_container
            .expect_has_binding::<dyn subjects::IUserManager>()
            .with(eq(None))
            .return_once(|_name| false)
            .once();

        mock_di_container
            .expect_set_binding::<dyn subjects::IUserManager>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder = BindingBuilder::<
            dyn subjects::IUserManager,
            mocks::blocking_di_container::MockDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(
            Rc::new(mock_di_container),
            mocks::MockDependencyHistory::new,
        );

        binding_builder.to_default_factory(&|_| {
            Box::new(move || {
                let user_manager: TransientPtr<dyn subjects::IUserManager> =
                    TransientPtr::new(subjects::UserManager::new());

                user_manager
            })
        })?;

        Ok(())
    }
}
