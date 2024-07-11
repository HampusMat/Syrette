//! Binding builder for types inside of a [`AsyncDIContainer`].
use std::any::type_name;
use std::marker::PhantomData;

use crate::di_container::asynchronous::binding::scope_configurator::AsyncBindingScopeConfigurator;
#[cfg(feature = "factory")]
use crate::di_container::asynchronous::binding::when_configurator::AsyncBindingWhenConfigurator;
use crate::di_container::BindingOptions;
use crate::errors::async_di_container::AsyncBindingBuilderError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);
use_double!(crate::di_container::asynchronous::AsyncDIContainer);

/// Alias for a threadsafe boxed function.
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
pub type BoxFn<Args, Return> = Box<(dyn Fn<Args, Output = Return> + Send + Sync)>;

/// Binding builder for type `Interface` inside a [`AsyncDIContainer`].
#[must_use = "No binding will be created if you don't use the binding builder"]
pub struct AsyncBindingBuilder<'di_container, Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    di_container: &'di_container mut AsyncDIContainer,
    dependency_history_factory: fn() -> DependencyHistory,

    interface_phantom: PhantomData<Interface>,
}

impl<'di_container, Interface> AsyncBindingBuilder<'di_container, Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    pub(crate) fn new(
        di_container: &'di_container mut AsyncDIContainer,
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
    /// associated [`AsyncDIContainer`].
    ///
    /// The scope of the binding is transient. But that can be changed by using the
    /// returned [`AsyncBindingScopeConfigurator`]
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::injectable;
    /// # use syrette::AsyncDIContainer;
    /// #
    /// # trait Foo: Send + Sync {}
    /// #
    /// # struct Bar {}
    /// #
    /// # #[injectable(Foo, async = true)]
    /// # impl Bar {
    /// #   fn new() -> Self
    /// #   {
    /// #       Self {}
    /// #   }
    /// # }
    /// #
    /// # impl Foo for Bar {}
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container.bind::<dyn Foo>().to::<Bar>()?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn to<Implementation>(
        self,
    ) -> Result<
        AsyncBindingScopeConfigurator<'di_container, Interface, Implementation>,
        AsyncBindingBuilderError,
    >
    where
        Implementation: AsyncInjectable<AsyncDIContainer>,
    {
        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let mut binding_scope_configurator = AsyncBindingScopeConfigurator::new(
            self.di_container,
            self.dependency_history_factory,
        );

        binding_scope_configurator.set_in_transient_scope();

        Ok(binding_scope_configurator)
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`AsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::AsyncDIContainer;
    /// # use syrette::ptr::TransientPtr;
    /// #
    /// # trait Foo: Send + Sync {}
    /// #
    /// # struct Bar
    /// # {
    /// #   num: i32,
    /// #   some_str: String
    /// # }
    /// #
    /// # impl Foo for Bar {}
    /// #
    /// # type FooFactory = dyn Fn(i32, String) -> TransientPtr<dyn Foo> + Send + Sync;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container.bind::<FooFactory>().to_factory(&|_| {
    ///     Box::new(|num, some_str| {
    ///         let bar = TransientPtr::new(Bar { num, some_str });
    ///
    ///         bar as TransientPtr<dyn Foo>
    ///     })
    /// })?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_factory<Args, Return, FactoryFunc>(
        self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<'di_container, Interface>,
        AsyncBindingBuilderError,
    >
    where
        Args: std::marker::Tuple + 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = Return> + Send + Sync,
        FactoryFunc: Fn(&AsyncDIContainer) -> BoxFn<Args, Return> + Send + Sync,
    {
        use crate::castable_function::threadsafe::ThreadsafeCastableFunction;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFunction::new(factory_func);

        self.di_container.set_binding::<Interface>(
            BindingOptions::new(),
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::Normal,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }

    /// Creates a binding of factory type `Interface` to a async factory inside of the
    /// associated [`AsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use std::time::Duration;
    /// #
    /// # use syrette::AsyncDIContainer;
    /// # use syrette::ptr::TransientPtr;
    /// # use syrette::future::BoxFuture;
    /// #
    /// # trait Foo: Send + Sync {}
    /// #
    /// # struct Bar
    /// # {
    /// #   num: i32,
    /// #   some_str: String
    /// # }
    /// #
    /// # impl Foo for Bar {}
    /// #
    /// # type FooFactory = dyn Fn(i32, String) -> BoxFuture<
    /// #   'static,
    /// #   TransientPtr<dyn Foo>
    /// # > + Send + Sync;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container.bind::<FooFactory>().to_async_factory(&|_| {
    ///     Box::new(|num, some_str| {
    ///         Box::pin(async move {
    ///             let bar = TransientPtr::new(Bar { num, some_str });
    ///
    ///             tokio::time::sleep(Duration::from_secs(2)).await;
    ///
    ///             bar as TransientPtr<dyn Foo>
    ///         })
    ///     })
    /// })?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_async_factory<Args, Return, FactoryFunc>(
        self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<'di_container, Interface>,
        AsyncBindingBuilderError,
    >
    where
        Args: std::marker::Tuple + 'static,
        Return: 'static + ?Sized,
        Interface:
            Fn<Args, Output = crate::future::BoxFuture<'static, Return>> + Send + Sync,
        FactoryFunc: Fn(
                &AsyncDIContainer,
            ) -> BoxFn<Args, crate::future::BoxFuture<'static, Return>>
            + Send
            + Sync,
    {
        use crate::castable_function::threadsafe::ThreadsafeCastableFunction;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFunction::new(factory_func);

        self.di_container.set_binding::<Interface>(
            BindingOptions::new(),
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::Normal,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }

    /// Creates a binding of type `Interface` to a factory that takes no arguments
    /// inside of the associated [`AsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::AsyncDIContainer;
    /// # use syrette::ptr::TransientPtr;
    /// #
    /// # trait Foo: Send + Sync {}
    /// #
    /// # struct Bar
    /// # {
    /// #   num: i32,
    /// #   some_str: String
    /// # }
    /// #
    /// # impl Foo for Bar {}
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container.bind::<dyn Foo>().to_default_factory(&|_| {
    ///     Box::new(|| {
    ///         let bar = TransientPtr::new(Bar {
    ///             num: 42,
    ///             some_str: "hello".to_string(),
    ///         });
    ///
    ///         bar as TransientPtr<dyn Foo>
    ///     })
    /// })?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_default_factory<Return, FactoryFunc>(
        self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<'di_container, Interface>,
        AsyncBindingBuilderError,
    >
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn(&AsyncDIContainer) -> BoxFn<(), crate::ptr::TransientPtr<Return>>
            + Send
            + Sync,
    {
        use crate::castable_function::threadsafe::ThreadsafeCastableFunction;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFunction::new(factory_func);

        self.di_container.set_binding::<Interface>(
            BindingOptions::new(),
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::Default,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }

    /// Creates a binding of factory type `Interface` to a async factory inside of the
    /// associated [`AsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use std::time::Duration;
    /// #
    /// # use syrette::AsyncDIContainer;
    /// # use syrette::ptr::TransientPtr;
    /// #
    /// # trait Foo: Send + Sync {}
    /// #
    /// # struct Bar
    /// # {
    /// #   num: i32,
    /// #   some_str: String
    /// # }
    /// #
    /// # impl Foo for Bar {}
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container
    ///     .bind::<dyn Foo>()
    ///     .to_async_default_factory(&|_| {
    ///         Box::new(|| {
    ///             Box::pin(async {
    ///                 let bar = TransientPtr::new(Bar {
    ///                     num: 42,
    ///                     some_str: "hello".to_string(),
    ///                 });
    ///
    ///                 tokio::time::sleep(Duration::from_secs(1)).await;
    ///
    ///                 bar as TransientPtr<dyn Foo>
    ///             })
    ///         })
    ///     })?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub fn to_async_default_factory<Return, FactoryFunc>(
        self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<'di_container, Interface>,
        AsyncBindingBuilderError,
    >
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn(&AsyncDIContainer) -> BoxFn<(), crate::future::BoxFuture<'static, Return>>
            + Send
            + Sync,
    {
        use crate::castable_function::threadsafe::ThreadsafeCastableFunction;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self
            .di_container
            .has_binding::<Interface>(BindingOptions::new())
        {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFunction::new(factory_func);

        self.di_container.set_binding::<Interface>(
            BindingOptions::new(),
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::AsyncDefault,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }
}

#[cfg(test)]
mod tests
{
    use mockall::predicate::eq;

    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::di_container::asynchronous::MockAsyncDIContainer;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn can_bind_to()
    {
        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_has_binding::<dyn subjects_async::IUserManager>()
            .with(eq(BindingOptions::new()))
            .return_once(|_name| false)
            .once();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder =
            AsyncBindingBuilder::<dyn subjects_async::IUserManager>::new(
                &mut di_container_mock,
                MockDependencyHistory::new,
            );

        binding_builder.to::<subjects_async::UserManager>().unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_bind_to_factory()
    {
        use crate::ptr::TransientPtr;

        type IUserManagerFactory = dyn Fn(
                String,
                i32,
                subjects_async::Number,
            ) -> TransientPtr<dyn subjects_async::IUserManager>
            + Send
            + Sync;

        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_has_binding::<IUserManagerFactory>()
            .with(eq(BindingOptions::new()))
            .return_once(|_name| false)
            .once();

        di_container_mock
            .expect_set_binding::<IUserManagerFactory>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder = AsyncBindingBuilder::<IUserManagerFactory>::new(
            &mut di_container_mock,
            MockDependencyHistory::new,
        );

        binding_builder
            .to_factory(&|_| {
                Box::new(|_text, _num, _number| {
                    let user_manager: TransientPtr<dyn subjects_async::IUserManager> =
                        TransientPtr::new(subjects_async::UserManager::new());

                    user_manager
                })
            })
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_bind_to_async_factory()
    {
        use crate::future::BoxFuture;
        use crate::ptr::TransientPtr;
        use crate::test_utils::async_closure;

        #[rustfmt::skip]
        type IUserManagerFactory = dyn Fn(String) -> BoxFuture<
            'static,
            TransientPtr<dyn subjects_async::IUserManager>
        > + Send + Sync;

        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_has_binding::<IUserManagerFactory>()
            .with(eq(BindingOptions::new()))
            .return_once(|_name| false)
            .once();

        di_container_mock
            .expect_set_binding::<IUserManagerFactory>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder = AsyncBindingBuilder::<IUserManagerFactory>::new(
            &mut di_container_mock,
            MockDependencyHistory::new,
        );

        binding_builder
            .to_async_factory(&|_| {
                async_closure!(|_text| {
                    let user_manager: TransientPtr<dyn subjects_async::IUserManager> =
                        TransientPtr::new(subjects_async::UserManager::new());

                    user_manager
                })
            })
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_bind_to_default_factory()
    {
        use crate::ptr::TransientPtr;

        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_has_binding::<dyn subjects_async::IUserManager>()
            .with(eq(BindingOptions::new()))
            .return_once(|_name| false)
            .once();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder =
            AsyncBindingBuilder::<dyn subjects_async::IUserManager>::new(
                &mut di_container_mock,
                MockDependencyHistory::new,
            );

        binding_builder
            .to_default_factory(&|_| {
                Box::new(|| {
                    let user_manager: TransientPtr<dyn subjects_async::IUserManager> =
                        TransientPtr::new(subjects_async::UserManager::new());

                    user_manager
                })
            })
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_bind_to_async_default_factory()
    {
        use crate::ptr::TransientPtr;
        use crate::test_utils::async_closure;

        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_has_binding::<dyn subjects_async::IUserManager>()
            .with(eq(BindingOptions::new()))
            .return_once(|_name| false)
            .once();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|binding_options, _provider| binding_options.name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder =
            AsyncBindingBuilder::<dyn subjects_async::IUserManager>::new(
                &mut di_container_mock,
                MockDependencyHistory::new,
            );

        binding_builder
            .to_async_default_factory(&|_| {
                async_closure!(|| {
                    let user_manager: TransientPtr<dyn subjects_async::IUserManager> =
                        TransientPtr::new(subjects_async::UserManager::new());

                    user_manager
                })
            })
            .unwrap();
    }
}
