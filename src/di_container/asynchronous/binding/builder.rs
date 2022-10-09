//! Binding builder for types inside of a [`AsyncDIContainer`].
use std::any::type_name;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::di_container::asynchronous::binding::scope_configurator::AsyncBindingScopeConfigurator;
#[cfg(feature = "factory")]
use crate::di_container::asynchronous::binding::when_configurator::AsyncBindingWhenConfigurator;
use crate::errors::async_di_container::AsyncBindingBuilderError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::AsyncDIContainer;

/// Alias for a threadsafe boxed function.
#[cfg(feature = "factory")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
pub type BoxFn<Args, Return> = Box<(dyn Fn<Args, Output = Return> + Send + Sync)>;

/// Binding builder for type `Interface` inside a [`AsyncDIContainer`].
pub struct AsyncBindingBuilder<Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    di_container: Arc<AsyncDIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> AsyncBindingBuilder<Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    pub(crate) fn new(di_container: Arc<AsyncDIContainer>) -> Self
    {
        Self {
            di_container,
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
    /// # use syrette::{AsyncDIContainer, injectable};
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
    pub async fn to<Implementation>(
        &self,
    ) -> Result<
        AsyncBindingScopeConfigurator<Interface, Implementation>,
        AsyncBindingBuilderError,
    >
    where
        Implementation: AsyncInjectable,
    {
        {
            let bindings_lock = self.di_container.bindings.lock().await;

            if bindings_lock.has::<Interface>(None) {
                return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let binding_scope_configurator =
            AsyncBindingScopeConfigurator::new(self.di_container.clone());

        binding_scope_configurator.in_transient_scope().await;

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
    /// # use syrette::{AsyncDIContainer, factory};
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
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_factory<Args, Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = Return> + Send + Sync,
        FactoryFunc:
            Fn<(Arc<AsyncDIContainer>,), Output = BoxFn<Args, Return>> + Send + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        let mut bindings_lock = self.di_container.bindings.lock().await;

        if bindings_lock.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        bindings_lock.set::<Interface>(
            None,
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::Normal,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
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
    /// # use syrette::{AsyncDIContainer, factory, async_closure};
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
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_async_factory<Args, Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface:
            Fn<Args, Output = crate::future::BoxFuture<'static, Return>> + Send + Sync,
        FactoryFunc: Fn<
                (Arc<AsyncDIContainer>,),
                Output = BoxFn<Args, crate::future::BoxFuture<'static, Return>>,
            > + Send
            + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        let mut bindings_lock = self.di_container.bindings.lock().await;

        if bindings_lock.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        bindings_lock.set::<Interface>(
            None,
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::Normal,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
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
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
                (Arc<AsyncDIContainer>,),
                Output = BoxFn<(), crate::ptr::TransientPtr<Return>>,
            > + Send
            + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        let mut bindings_lock = self.di_container.bindings.lock().await;

        if bindings_lock.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        bindings_lock.set::<Interface>(
            None,
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::Default,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
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
    /// # use syrette::{AsyncDIContainer, async_closure};
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
    #[cfg(feature = "factory")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "factory")))]
    pub async fn to_async_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
                (Arc<AsyncDIContainer>,),
                Output = BoxFn<(), crate::future::BoxFuture<'static, Return>>,
            > + Send
            + Sync,
    {
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
        use crate::provider::r#async::AsyncFactoryVariant;

        let mut bindings_lock = self.di_container.bindings.lock().await;

        if bindings_lock.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        bindings_lock.set::<Interface>(
            None,
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
                AsyncFactoryVariant::AsyncDefault,
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use super::*;
    use crate::ptr::TransientPtr;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn can_bind_to() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<dyn subjects_async::IUserManager>()
            .to::<subjects_async::UserManager>()
            .await?;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_bind_to_transient() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<dyn subjects_async::IUserManager>()
            .to::<subjects_async::UserManager>()
            .await?
            .in_transient_scope()
            .await;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_bind_to_transient_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<dyn subjects_async::IUserManager>()
            .to::<subjects_async::UserManager>()
            .await?
            .in_transient_scope()
            .await
            .when_named("regular")
            .await?;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_bind_to_singleton() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<dyn subjects_async::IUserManager>()
            .to::<subjects_async::UserManager>()
            .await?
            .in_singleton_scope()
            .await?;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }

    #[tokio::test]
    async fn can_bind_to_singleton_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<dyn subjects_async::IUserManager>()
            .to::<subjects_async::UserManager>()
            .await?
            .in_singleton_scope()
            .await?
            .when_named("cool")
            .await?;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }

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

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_bind_to_factory_when_named() -> Result<(), Box<dyn Error>>
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
            .await?
            .when_named("awesome")
            .await?;

        {
            assert_eq!(di_container.bindings.lock().await.count(), 1);
        }

        Ok(())
    }
}
