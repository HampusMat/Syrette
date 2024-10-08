//! Binding builder for types inside of a [`DIContainer`].
use std::any::type_name;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::castable_function::CastableFunction;
use crate::di_container::blocking::binding::scope_configurator::BindingScopeConfigurator;
use crate::di_container::blocking::binding::when_configurator::BindingWhenConfigurator;
use crate::di_container::BindingOptions;
use crate::errors::di_container::BindingBuilderError;
use crate::interfaces::injectable::Injectable;
use crate::provider::blocking::{FunctionProvider, ProvidableFunctionKind};
use crate::ptr::TransientPtr;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);
use_double!(crate::di_container::blocking::DIContainer);

/// Binding builder for type `Interface` inside a [`DIContainer`].
#[must_use = "No binding will be created if you don't use the binding builder"]
pub struct BindingBuilder<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: &'di_container mut DIContainer,
    dependency_history_factory: fn() -> DependencyHistory,

    interface_phantom: PhantomData<Interface>,
}

impl<'di_container, Interface> BindingBuilder<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    pub(crate) fn new(
        di_container: &'di_container mut DIContainer,
        dependency_history_factory: fn() -> DependencyHistory,
    ) -> Self
    {
        Self {
            di_container,
            dependency_history_factory,
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
        self,
    ) -> Result<
        BindingScopeConfigurator<'di_container, Interface, Implementation>,
        BindingBuilderError,
    >
    where
        Implementation: Injectable<DIContainer>,
    {
        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >()));
        }

        let mut binding_scope_configurator = BindingScopeConfigurator::new(
            self.di_container,
            self.dependency_history_factory,
        );

        binding_scope_configurator.set_in_transient_scope();

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
    /// # use syrette::DIContainer;
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
    /// # type ICustomerFactory = dyn Fn(String, u32) -> TransientPtr<dyn ICustomer>;
    /// #
    /// # type ICustomerIDFactory = dyn Fn(u32) -> TransientPtr<dyn ICustomerID>;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = DIContainer::new();
    /// #
    /// di_container
    ///     .bind::<ICustomerFactory>()
    ///     .to_factory(&|context| {
    ///         let customer_id_factory = context
    ///             .get::<ICustomerIDFactory>()
    ///             .unwrap()
    ///             .factory()
    ///             .unwrap();
    ///
    ///         Box::new(move |name, id| {
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
        self,
        factory_func: &'static Func,
    ) -> Result<BindingWhenConfigurator<'di_container, Interface>, BindingBuilderError>
    where
        Args: std::marker::Tuple + 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = crate::ptr::TransientPtr<Return>>,
        Func: Fn(&DIContainer) -> Box<Interface>,
    {
        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >()));
        }

        let factory_impl = CastableFunction::new(factory_func);

        self.di_container.set_binding::<Interface>(
            BindingOptions::new(),
            Box::new(FunctionProvider::new(
                Rc::new(factory_impl),
                ProvidableFunctionKind::UserCalled,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container))
    }

    /// Creates a binding of type `Interface` to a value resolved using the given
    /// function.
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::DIContainer;
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
    /// di_container.bind::<dyn IBuffer>().to_dynamic_value(&|_| {
    ///     Box::new(|| TransientPtr::new(Buffer::<BUFFER_SIZE>::new()))
    /// });
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_dynamic_value<Func>(
        self,
        func: &'static Func,
    ) -> Result<BindingWhenConfigurator<'di_container, Interface>, BindingBuilderError>
    where
        Func: Fn(&DIContainer) -> TransientPtr<dyn Fn() -> TransientPtr<Interface>>,
    {
        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >()));
        }

        let castable_func = CastableFunction::new(func);

        self.di_container.set_binding::<Interface>(
            BindingOptions::new(),
            Box::new(FunctionProvider::new(
                Rc::new(castable_func),
                ProvidableFunctionKind::Instant,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container))
    }
}

#[cfg(test)]
mod tests
{
    use mockall::predicate::eq;

    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::di_container::blocking::MockDIContainer;
    use crate::test_utils::subjects;

    #[test]
    fn can_bind_to()
    {
        let mut mock_di_container = MockDIContainer::new();

        mock_di_container
            .expect_has_binding::<dyn subjects::INumber>()
            .with(eq(BindingOptions::new()))
            .return_once(|_options| false)
            .once();

        mock_di_container
            .expect_set_binding::<dyn subjects::INumber>()
            .withf(|options, _provider| options.name.is_none())
            .return_once(|_options, _provider| ())
            .once();

        let binding_builder = BindingBuilder::<dyn subjects::INumber>::new(
            &mut mock_di_container,
            MockDependencyHistory::new,
        );

        binding_builder.to::<subjects::Number>().unwrap();
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory()
    {
        use crate::ptr::TransientPtr;

        type IUserManagerFactory =
            dyn Fn(i32, String) -> TransientPtr<dyn subjects::IUserManager>;

        let mut mock_di_container = MockDIContainer::new();

        mock_di_container
            .expect_has_binding::<IUserManagerFactory>()
            .with(eq(BindingOptions::new()))
            .return_once(|_| false)
            .once();

        mock_di_container
            .expect_set_binding::<IUserManagerFactory>()
            .withf(|options, _provider| options.name.is_none())
            .return_once(|_, _provider| ())
            .once();

        let binding_builder = BindingBuilder::<IUserManagerFactory>::new(
            &mut mock_di_container,
            MockDependencyHistory::new,
        );

        binding_builder
            .to_factory(&|_| {
                Box::new(move |_num, _text| {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

                    user_manager
                })
            })
            .unwrap();
    }

    #[test]
    fn can_bind_to_dynamic_value()
    {
        use crate::ptr::TransientPtr;

        let mut mock_di_container = MockDIContainer::new();

        mock_di_container
            .expect_has_binding::<dyn subjects::IUserManager>()
            .with(eq(BindingOptions::new()))
            .return_once(|_| false)
            .once();

        mock_di_container
            .expect_set_binding::<dyn subjects::IUserManager>()
            .withf(|options, _provider| options.name.is_none())
            .return_once(|_, _provider| ())
            .once();

        let binding_builder = BindingBuilder::<dyn subjects::IUserManager>::new(
            &mut mock_di_container,
            MockDependencyHistory::new,
        );

        binding_builder
            .to_dynamic_value(&|_| {
                Box::new(move || {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

                    user_manager
                })
            })
            .unwrap();
    }
}
