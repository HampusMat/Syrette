//! Binding builder for types inside of a [`DIContainer`].
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::binding::scope_configurator::BindingScopeConfigurator;
#[cfg(feature = "factory")]
use crate::di_container::blocking::binding::when_configurator::BindingWhenConfigurator;
use crate::di_container::blocking::DIContainer;
use crate::errors::di_container::BindingBuilderError;
use crate::interfaces::injectable::Injectable;

/// Binding builder for type `Interface` inside a [`DIContainer`].
pub struct BindingBuilder<Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: Rc<DIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> BindingBuilder<Interface>
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

    /// Creates a binding of type `Interface` to type `Implementation` inside of the
    /// associated [`DIContainer`].
    ///
    /// The scope of the binding is transient. But that can be changed by using the
    /// returned [`BindingScopeConfigurator`]
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{DIContainer, injectable};
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
    pub fn to<Implementation>(
        &self,
    ) -> Result<BindingScopeConfigurator<Interface, Implementation>, BindingBuilderError>
    where
        Implementation: Injectable,
    {
        {
            let bindings = self.di_container.bindings.borrow();

            if bindings.has::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let binding_scope_configurator =
            BindingScopeConfigurator::new(self.di_container.clone());

        binding_scope_configurator.in_transient_scope();

        Ok(binding_scope_configurator)
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`DIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{DIContainer, factory};
    /// # use syrette::ptr::TransientPtr;
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
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_factory<Args, Return, Func>(
        &self,
        factory_func: &'static Func,
    ) -> Result<BindingWhenConfigurator<Interface>, BindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = crate::ptr::TransientPtr<Return>>,
        Func: Fn<(std::rc::Rc<DIContainer>,), Output = Box<Interface>>,
    {
        use crate::castable_factory::blocking::CastableFactory;

        {
            let bindings = self.di_container.bindings.borrow();

            if bindings.has::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        let factory_impl = CastableFactory::new(factory_func);

        bindings_mut.set::<Interface>(
            None,
            Box::new(crate::provider::blocking::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
                false,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
    }

    /// Creates a binding of type `Interface` to a factory that takes no arguments
    /// inside of the associated [`DIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{DIContainer, factory};
    /// # use syrette::ptr::TransientPtr;
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
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<BindingWhenConfigurator<Interface>, BindingBuilderError>
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
            (Rc<DIContainer>,),
            Output = crate::ptr::TransientPtr<
                dyn Fn<(), Output = crate::ptr::TransientPtr<Return>>,
            >,
        >,
    {
        use crate::castable_factory::blocking::CastableFactory;

        {
            let bindings = self.di_container.bindings.borrow();

            if bindings.has::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        let factory_impl = CastableFactory::new(factory_func);

        bindings_mut.set::<Interface>(
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

    use super::*;
    use crate::ptr::TransientPtr;
    use crate::test_utils::subjects;

    #[test]
    fn can_bind_to() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_transient() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_transient_scope();

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_transient_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_transient_scope()
            .when_named("regular")?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_singleton() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_singleton_scope()?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_singleton_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_singleton_scope()?
            .when_named("cool")?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory() -> Result<(), Box<dyn Error>>
    {
        use crate as syrette;
        use crate::factory;

        #[factory]
        type IUserManagerFactory = dyn Fn() -> dyn subjects::IUserManager;

        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(move || {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

                    user_manager
                })
            })?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory_when_named() -> Result<(), Box<dyn Error>>
    {
        use crate as syrette;
        use crate::factory;

        #[factory]
        type IUserManagerFactory = dyn Fn() -> dyn subjects::IUserManager;

        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(move || {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

                    user_manager
                })
            })?
            .when_named("awesome")?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }
}
