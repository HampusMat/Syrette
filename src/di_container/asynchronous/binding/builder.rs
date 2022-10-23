//! Binding builder for types inside of a [`IAsyncDIContainer`].
//!
//! [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
use std::any::type_name;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::di_container::asynchronous::binding::scope_configurator::AsyncBindingScopeConfigurator;
#[cfg(feature = "factory")]
use crate::di_container::asynchronous::binding::when_configurator::AsyncBindingWhenConfigurator;
use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::async_di_container::AsyncBindingBuilderError;
use crate::interfaces::async_injectable::AsyncInjectable;

/// Alias for a threadsafe boxed function.
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
pub type BoxFn<Args, Return> = Box<(dyn Fn<Args, Output = Return> + Send + Sync)>;

/// Binding builder for type `Interface` inside a [`IAsyncDIContainer`].
///
/// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
pub struct AsyncBindingBuilder<Interface, DIContainerType>
where
    Interface: 'static + ?Sized + Send + Sync,
    DIContainerType: IAsyncDIContainer,
{
    di_container: Arc<DIContainerType>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface, DIContainerType> AsyncBindingBuilder<Interface, DIContainerType>
where
    Interface: 'static + ?Sized + Send + Sync,
    DIContainerType: IAsyncDIContainer,
{
    pub(crate) fn new(di_container: Arc<DIContainerType>) -> Self
    {
        Self {
            di_container,
            interface_phantom: PhantomData,
        }
    }

    /// Creates a binding of type `Interface` to type `Implementation` inside of the
    /// associated [`IAsyncDIContainer`].
    ///
    /// The scope of the binding is transient. But that can be changed by using the
    /// returned [`AsyncBindingScopeConfigurator`]
    ///
    /// # Errors
    /// Will return Err if the associated [`IAsyncDIContainer`] already have a binding for
    /// the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::injectable;
    /// # use syrette::di_container::asynchronous::prelude::*;
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
    /// di_container.bind::<dyn Foo>().to::<Bar>().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
    pub async fn to<Implementation>(
        &self,
    ) -> Result<
        AsyncBindingScopeConfigurator<Interface, Implementation, DIContainerType>,
        AsyncBindingBuilderError,
    >
    where
        Implementation: AsyncInjectable<DIContainerType>,
    {
        if self.di_container.has_binding::<Interface>(None).await {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let binding_scope_configurator =
            AsyncBindingScopeConfigurator::new(self.di_container.clone());

        binding_scope_configurator.in_transient_scope().await;

        Ok(binding_scope_configurator)
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`IAsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`IAsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::{factory};
    /// # use syrette::di_container::asynchronous::prelude::*;
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
    /// # #[factory(threadsafe = true)]
    /// # type FooFactory = dyn Fn(i32, String) -> dyn Foo;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container
    ///     .bind::<FooFactory>()
    ///     .to_factory(&|_| {
    ///         Box::new(|num, some_str| {
    ///             let bar = TransientPtr::new(Bar { num, some_str });
    ///
    ///             bar as TransientPtr<dyn Foo>
    ///         })
    ///     })
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_factory<Args, Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<Interface, DIContainerType>,
        AsyncBindingBuilderError,
    >
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = Return> + Send + Sync,
        FactoryFunc:
            Fn<(Arc<DIContainerType>,), Output = BoxFn<Args, Return>> + Send + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self.di_container.has_binding::<Interface>(None).await {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        self.di_container
            .set_binding::<Interface>(
                None,
                Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                    crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                    AsyncFactoryVariant::Normal,
                )),
            )
            .await;

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }

    /// Creates a binding of factory type `Interface` to a async factory inside of the
    /// associated [`IAsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`IAsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use std::time::Duration;
    /// #
    /// # use syrette::{factory, async_closure};
    /// # use syrette::di_container::asynchronous::prelude::*;
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
    /// # #[factory(async = true)]
    /// # type FooFactory = dyn Fn(i32, String) -> dyn Foo;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>>
    /// # {
    /// # let mut di_container = AsyncDIContainer::new();
    /// #
    /// di_container
    ///     .bind::<FooFactory>()
    ///     .to_async_factory(&|_| {
    ///         async_closure!(|num, some_str| {
    ///             let bar = TransientPtr::new(Bar { num, some_str });
    ///
    ///             tokio::time::sleep(Duration::from_secs(2)).await;
    ///
    ///             bar as TransientPtr<dyn Foo>
    ///         })
    ///     })
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_async_factory<Args, Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<Interface, DIContainerType>,
        AsyncBindingBuilderError,
    >
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface:
            Fn<Args, Output = crate::future::BoxFuture<'static, Return>> + Send + Sync,
        FactoryFunc: Fn<
                (Arc<DIContainerType>,),
                Output = BoxFn<Args, crate::future::BoxFuture<'static, Return>>,
            > + Send
            + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self.di_container.has_binding::<Interface>(None).await {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        self.di_container
            .set_binding::<Interface>(
                None,
                Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                    crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                    AsyncFactoryVariant::Normal,
                )),
            )
            .await;

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }

    /// Creates a binding of type `Interface` to a factory that takes no arguments
    /// inside of the associated [`IAsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`IAsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// #
    /// # use syrette::di_container::asynchronous::prelude::*;
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
    ///     .to_default_factory(&|_| {
    ///         Box::new(|| {
    ///             let bar = TransientPtr::new(Bar {
    ///                 num: 42,
    ///                 some_str: "hello".to_string(),
    ///             });
    ///
    ///             bar as TransientPtr<dyn Foo>
    ///         })
    ///     })
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<Interface, DIContainerType>,
        AsyncBindingBuilderError,
    >
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
                (Arc<DIContainerType>,),
                Output = BoxFn<(), crate::ptr::TransientPtr<Return>>,
            > + Send
            + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self.di_container.has_binding::<Interface>(None).await {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        self.di_container
            .set_binding::<Interface>(
                None,
                Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                    crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                    AsyncFactoryVariant::Default,
                )),
            )
            .await;

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }

    /// Creates a binding of factory type `Interface` to a async factory inside of the
    /// associated [`IAsyncDIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`IAsyncDIContainer`] already have a binding
    /// for the interface.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use std::time::Duration;
    /// #
    /// # use syrette::async_closure;
    /// # use syrette::di_container::asynchronous::prelude::*;
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
    ///         async_closure!(|| {
    ///             let bar = TransientPtr::new(Bar {
    ///                 num: 42,
    ///                 some_str: "hello".to_string(),
    ///             });
    ///
    ///             tokio::time::sleep(Duration::from_secs(1)).await;
    ///
    ///             bar as TransientPtr<dyn Foo>
    ///         })
    ///     })
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`IAsyncDIContainer`]: crate::di_container::asynchronous::IAsyncDIContainer
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_async_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<
        AsyncBindingWhenConfigurator<Interface, DIContainerType>,
        AsyncBindingBuilderError,
    >
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
                (Arc<DIContainerType>,),
                Output = BoxFn<(), crate::future::BoxFuture<'static, Return>>,
            > + Send
            + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        if self.di_container.has_binding::<Interface>(None).await {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        self.di_container
            .set_binding::<Interface>(
                None,
                Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                    crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                    AsyncFactoryVariant::AsyncDefault,
                )),
            )
            .await;

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use mockall::predicate::eq;

    use super::*;
    use crate::test_utils::mocks::async_di_container::MockAsyncDIContainer;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn can_bind_to() -> Result<(), Box<dyn Error>>
    {
        let mut di_container_mock = MockAsyncDIContainer::new();

        di_container_mock
            .expect_has_binding::<dyn subjects_async::IUserManager>()
            .with(eq(None))
            .return_once(|_name| false)
            .once();

        di_container_mock
            .expect_set_binding::<dyn subjects_async::IUserManager>()
            .withf(|name, _provider| name.is_none())
            .return_once(|_name, _provider| ())
            .once();

        let binding_builder = AsyncBindingBuilder::<
            dyn subjects_async::IUserManager,
            MockAsyncDIContainer,
        >::new(Arc::new(di_container_mock));

        binding_builder.to::<subjects_async::UserManager>().await?;

        Ok(())
    }

    /*
    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_bind_to_factory() -> Result<(), Box<dyn Error>>
    {
        use crate as syrette;
        use crate::factory;

        #[factory(threadsafe = true)]
        type IUserManagerFactory = dyn Fn() -> dyn subjects_async::IUserManager;

        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(|| {
                    let user_manager: TransientPtr<dyn subjects_async::IUserManager> =
                        TransientPtr::new(subjects_async::UserManager::new());

                    user_manager
                })
            })
            .await?;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }
    */
}
