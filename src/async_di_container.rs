//! Asynchronous dependency injection container.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use std::error::Error;
//!
//! use syrette::{injectable, AsyncDIContainer};
//!
//! trait IDatabaseService: Send + Sync
//! {
//!     fn get_all_records(&self, table_name: String) -> HashMap<String, String>;
//! }
//!
//! struct DatabaseService {}
//!
//! #[injectable(IDatabaseService, async = true)]
//! impl DatabaseService
//! {
//!     fn new() -> Self
//!     {
//!         Self {}
//!     }
//! }
//!
//! impl IDatabaseService for DatabaseService
//! {
//!     fn get_all_records(&self, table_name: String) -> HashMap<String, String>
//!     {
//!         // Do stuff here
//!         HashMap::<String, String>::new()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>>
//! {
//!     let mut di_container = AsyncDIContainer::new();
//!
//!     di_container
//!         .bind::<dyn IDatabaseService>()
//!         .to::<DatabaseService>()
//!         .await?;
//!
//!     let database_service = di_container
//!         .get::<dyn IDatabaseService>()
//!         .await?
//!         .transient()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ---
//!
//! *This module is only available if Syrette is built with the "async" feature.*
use std::any::type_name;
use std::marker::PhantomData;
use std::sync::Arc;

use tokio::sync::Mutex;

#[cfg(feature = "factory")]
use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
use crate::di_container_binding_map::DIContainerBindingMap;
use crate::errors::async_di_container::{
    AsyncBindingBuilderError,
    AsyncBindingScopeConfiguratorError,
    AsyncBindingWhenConfiguratorError,
    AsyncDIContainerError,
};
use crate::future::BoxFuture;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::libs::intertrait::cast::error::CastError;
use crate::libs::intertrait::cast::{CastArc, CastBox};
use crate::provider::r#async::{
    AsyncProvidable,
    AsyncSingletonProvider,
    AsyncTransientTypeProvider,
    IAsyncProvider,
};
use crate::ptr::{SomeThreadsafePtr, ThreadsafeSingletonPtr, TransientPtr};

/// When configurator for a binding for type 'Interface' inside a [`AsyncDIContainer`].
pub struct AsyncBindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    di_container: Arc<AsyncDIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> AsyncBindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized + Send + Sync,
{
    fn new(di_container: Arc<AsyncDIContainer>) -> Self
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
    pub async fn when_named(
        &self,
        name: &'static str,
    ) -> Result<(), AsyncBindingWhenConfiguratorError>
    {
        let mut bindings_lock = self.di_container.bindings.lock().await;

        let binding = bindings_lock.remove::<Interface>(None).map_or_else(
            || {
                Err(AsyncBindingWhenConfiguratorError::BindingNotFound(
                    type_name::<Interface>(),
                ))
            },
            Ok,
        )?;

        bindings_lock.set::<Interface>(Some(name), binding);

        Ok(())
    }
}

/// Scope configurator for a binding for type 'Interface' inside a [`AsyncDIContainer`].
pub struct AsyncBindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable,
{
    di_container: Arc<AsyncDIContainer>,
    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<Interface, Implementation> AsyncBindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized + Send + Sync,
    Implementation: AsyncInjectable,
{
    fn new(di_container: Arc<AsyncDIContainer>) -> Self
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
    pub async fn in_transient_scope(&self) -> AsyncBindingWhenConfigurator<Interface>
    {
        let mut bindings_lock = self.di_container.bindings.lock().await;

        bindings_lock.set::<Interface>(
            None,
            Box::new(AsyncTransientTypeProvider::<Implementation>::new()),
        );

        AsyncBindingWhenConfigurator::new(self.di_container.clone())
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub async fn in_singleton_scope(
        &self,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingScopeConfiguratorError>
    {
        let singleton: ThreadsafeSingletonPtr<Implementation> =
            ThreadsafeSingletonPtr::from(
                Implementation::resolve(&self.di_container, Vec::new())
                    .await
                    .map_err(
                        AsyncBindingScopeConfiguratorError::SingletonResolveFailed,
                    )?,
            );

        let mut bindings_lock = self.di_container.bindings.lock().await;

        bindings_lock
            .set::<Interface>(None, Box::new(AsyncSingletonProvider::new(singleton)));

        Ok(AsyncBindingWhenConfigurator::new(self.di_container.clone()))
    }
}

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
    fn new(di_container: Arc<AsyncDIContainer>) -> Self
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
    /// *This function is only available if Syrette is built with the "factory" feature.*
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding for
    /// the interface.
    #[cfg(feature = "factory")]
    pub async fn to_factory<Args, Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = Return> + Send + Sync,
        FactoryFunc: Fn<
                (Arc<AsyncDIContainer>,),
                Output = Box<(dyn Fn<Args, Output = Return> + Send + Sync)>,
            > + Send
            + Sync,
    {
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
    /// *This function is only available if Syrette is built with the "factory" and
    /// "async" features.*
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding for
    /// the interface.
    #[cfg(all(feature = "factory", feature = "async"))]
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
                Output = Box<
                    (dyn Fn<Args, Output = crate::future::BoxFuture<'static, Return>>
                         + Send
                         + Sync),
                >,
            > + Send
            + Sync,
    {
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
    /// *This function is only available if Syrette is built with the "factory" feature.*
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding for
    /// the interface.
    #[cfg(feature = "factory")]
    pub async fn to_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
                (Arc<AsyncDIContainer>,),
                Output = Box<
                    (dyn Fn<(), Output = crate::ptr::TransientPtr<Return>> + Send + Sync),
                >,
            > + Send
            + Sync,
    {
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
    /// *This function is only available if Syrette is built with the "factory" and
    /// "async" features.*
    ///
    /// # Errors
    /// Will return Err if the associated [`AsyncDIContainer`] already have a binding for
    /// the interface.
    #[cfg(all(feature = "factory", feature = "async"))]
    pub async fn to_async_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
                (Arc<AsyncDIContainer>,),
                Output = Box<
                    (dyn Fn<(), Output = crate::future::BoxFuture<'static, Return>>
                         + Send
                         + Sync),
                >,
            > + Send
            + Sync,
    {
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

/// Dependency injection container.
pub struct AsyncDIContainer
{
    bindings: Mutex<DIContainerBindingMap<dyn IAsyncProvider>>,
}

impl AsyncDIContainer
{
    /// Returns a new `AsyncDIContainer`.
    #[must_use]
    pub fn new() -> Arc<Self>
    {
        Arc::new(Self {
            bindings: Mutex::new(DIContainerBindingMap::new()),
        })
    }

    /// Returns a new [`AsyncBindingBuilder`] for the given interface.
    #[must_use]
    pub fn bind<Interface>(self: &mut Arc<Self>) -> AsyncBindingBuilder<Interface>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        AsyncBindingBuilder::<Interface>::new(self.clone())
    }

    /// Returns the type bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for fails
    /// - Casting the binding for fails
    pub async fn get<Interface>(
        self: &Arc<Self>,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        self.get_bound::<Interface>(Vec::new(), None).await
    }

    /// Returns the type bound with `Interface` and the specified name.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` with name `name` exists
    /// - Resolving the binding fails
    /// - Casting the binding for fails
    pub async fn get_named<Interface>(
        self: &Arc<Self>,
        name: &'static str,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        self.get_bound::<Interface>(Vec::new(), Some(name)).await
    }

    #[doc(hidden)]
    pub async fn get_bound<Interface>(
        self: &Arc<Self>,
        dependency_history: Vec<&'static str>,
        name: Option<&'static str>,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        let binding_providable = self
            .get_binding_providable::<Interface>(name, dependency_history)
            .await?;

        self.handle_binding_providable(binding_providable).await
    }

    async fn handle_binding_providable<Interface>(
        self: &Arc<Self>,
        binding_providable: AsyncProvidable,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        match binding_providable {
            AsyncProvidable::Transient(transient_binding) => {
                Ok(SomeThreadsafePtr::Transient(
                    transient_binding.cast::<Interface>().map_err(|_| {
                        AsyncDIContainerError::CastFailed {
                            interface: type_name::<Interface>(),
                            binding_kind: "transient",
                        }
                    })?,
                ))
            }
            AsyncProvidable::Singleton(singleton_binding) => {
                Ok(SomeThreadsafePtr::ThreadsafeSingleton(
                    singleton_binding
                        .cast::<Interface>()
                        .map_err(|err| match err {
                            CastError::NotArcCastable(_) => {
                                AsyncDIContainerError::InterfaceNotAsync(type_name::<
                                    Interface,
                                >(
                                ))
                            }
                            CastError::CastFailed { from: _, to: _ } => {
                                AsyncDIContainerError::CastFailed {
                                    interface: type_name::<Interface>(),
                                    binding_kind: "singleton",
                                }
                            }
                        })?,
                ))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::Factory(factory_binding) => {
                use crate::interfaces::factory::IThreadsafeFactory;

                let factory = factory_binding
                    .cast::<dyn IThreadsafeFactory<(Arc<AsyncDIContainer>,), Interface>>()
                    .map_err(|err| match err {
                        CastError::NotArcCastable(_) => {
                            AsyncDIContainerError::InterfaceNotAsync(
                                type_name::<Interface>(),
                            )
                        }
                        CastError::CastFailed { from: _, to: _ } => {
                            AsyncDIContainerError::CastFailed {
                                interface: type_name::<Interface>(),
                                binding_kind: "factory",
                            }
                        }
                    })?;

                Ok(SomeThreadsafePtr::ThreadsafeFactory(
                    factory(self.clone()).into(),
                ))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::DefaultFactory(binding) => {
                use crate::interfaces::factory::IThreadsafeFactory;

                let default_factory = Self::cast_factory_binding::<
                    dyn IThreadsafeFactory<
                        (Arc<AsyncDIContainer>,),
                        dyn Fn<(), Output = TransientPtr<Interface>> + Send + Sync,
                    >,
                >(binding, "default factory")?;

                Ok(SomeThreadsafePtr::Transient(default_factory(self.clone())()))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::AsyncDefaultFactory(binding) => {
                use crate::interfaces::factory::IThreadsafeFactory;

                let async_default_factory = Self::cast_factory_binding::<
                    dyn IThreadsafeFactory<
                        (Arc<AsyncDIContainer>,),
                        dyn Fn<(), Output = BoxFuture<'static, TransientPtr<Interface>>>
                            + Send
                            + Sync,
                    >,
                >(
                    binding, "async default factory"
                )?;

                Ok(SomeThreadsafePtr::Transient(
                    async_default_factory(self.clone())().await,
                ))
            }
        }
    }

    #[cfg(feature = "factory")]
    fn cast_factory_binding<Type: 'static + ?Sized>(
        factory_binding: Arc<dyn crate::interfaces::any_factory::AnyThreadsafeFactory>,
        binding_kind: &'static str,
    ) -> Result<Arc<Type>, AsyncDIContainerError>
    {
        factory_binding.cast::<Type>().map_err(|err| match err {
            CastError::NotArcCastable(_) => {
                AsyncDIContainerError::InterfaceNotAsync(type_name::<Type>())
            }
            CastError::CastFailed { from: _, to: _ } => {
                AsyncDIContainerError::CastFailed {
                    interface: type_name::<Type>(),
                    binding_kind,
                }
            }
        })
    }

    async fn get_binding_providable<Interface>(
        self: &Arc<Self>,
        name: Option<&'static str>,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        let provider;

        {
            let bindings_lock = self.bindings.lock().await;

            provider = bindings_lock
                .get::<Interface>(name)
                .map_or_else(
                    || {
                        Err(AsyncDIContainerError::BindingNotFound {
                            interface: type_name::<Interface>(),
                            name,
                        })
                    },
                    Ok,
                )?
                .clone();
        }

        provider
            .provide(self, dependency_history)
            .await
            .map_err(|err| AsyncDIContainerError::BindingResolveFailed {
                reason: err,
                interface: type_name::<Interface>(),
            })
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use async_trait::async_trait;
    use mockall::mock;

    use super::*;
    use crate::errors::injectable::InjectableError;
    use crate::ptr::TransientPtr;

    mod subjects
    {
        //! Test subjects.

        use std::fmt::Debug;
        use std::sync::Arc;

        use async_trait::async_trait;
        use syrette_macros::declare_interface;

        use super::AsyncDIContainer;
        use crate::interfaces::async_injectable::AsyncInjectable;
        use crate::ptr::TransientPtr;

        pub trait IUserManager: Send + Sync
        {
            fn add_user(&self, user_id: i128);

            fn remove_user(&self, user_id: i128);
        }

        pub struct UserManager {}

        impl UserManager
        {
            pub fn new() -> Self
            {
                Self {}
            }
        }

        impl IUserManager for UserManager
        {
            fn add_user(&self, _user_id: i128)
            {
                // ...
            }

            fn remove_user(&self, _user_id: i128)
            {
                // ...
            }
        }

        use crate as syrette;

        declare_interface!(UserManager -> IUserManager);

        #[async_trait]
        impl AsyncInjectable for UserManager
        {
            async fn resolve(
                _: &Arc<AsyncDIContainer>,
                _dependency_history: Vec<&'static str>,
            ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
            where
                Self: Sized,
            {
                Ok(TransientPtr::new(Self::new()))
            }
        }

        pub trait INumber: Send + Sync
        {
            fn get(&self) -> i32;

            fn set(&mut self, number: i32);
        }

        impl PartialEq for dyn INumber
        {
            fn eq(&self, other: &Self) -> bool
            {
                self.get() == other.get()
            }
        }

        impl Debug for dyn INumber
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
            {
                f.write_str(format!("{}", self.get()).as_str())
            }
        }

        pub struct Number
        {
            pub num: i32,
        }

        impl Number
        {
            pub fn new() -> Self
            {
                Self { num: 0 }
            }
        }

        impl INumber for Number
        {
            fn get(&self) -> i32
            {
                self.num
            }

            fn set(&mut self, number: i32)
            {
                self.num = number;
            }
        }

        declare_interface!(Number -> INumber, async = true);

        #[async_trait]
        impl AsyncInjectable for Number
        {
            async fn resolve(
                _: &Arc<AsyncDIContainer>,
                _dependency_history: Vec<&'static str>,
            ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
            where
                Self: Sized,
            {
                Ok(TransientPtr::new(Self::new()))
            }
        }
    }

    #[tokio::test]
    async fn can_bind_to() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()
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
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()
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
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()
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
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()
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
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()
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
        type IUserManagerFactory = dyn Fn() -> dyn subjects::IUserManager;

        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(|| {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

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
        type IUserManagerFactory = dyn Fn() -> dyn subjects::IUserManager;

        let mut di_container = AsyncDIContainer::new();

        {
            assert_eq!(di_container.bindings.lock().await.count(), 0);
        }

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(|| {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

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

    #[tokio::test]
    async fn can_get() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &Arc<AsyncDIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;

                fn do_clone(&self) -> Box<dyn IAsyncProvider>;
            }
        }

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            let mut inner_mock_provider = MockProvider::new();

            inner_mock_provider.expect_provide().returning(|_, _| {
                Ok(AsyncProvidable::Transient(TransientPtr::new(
                    subjects::UserManager::new(),
                )))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<dyn subjects::IUserManager>(None, Box::new(mock_provider));
        }

        di_container
            .get::<dyn subjects::IUserManager>()
            .await?
            .transient()?;

        Ok(())
    }

    #[tokio::test]
    async fn can_get_named() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &Arc<AsyncDIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;

                fn do_clone(&self) -> Box<dyn IAsyncProvider>;
            }
        }

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            let mut inner_mock_provider = MockProvider::new();

            inner_mock_provider.expect_provide().returning(|_, _| {
                Ok(AsyncProvidable::Transient(TransientPtr::new(
                    subjects::UserManager::new(),
                )))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<dyn subjects::IUserManager>(
                    Some("special"),
                    Box::new(mock_provider),
                );
        }

        di_container
            .get_named::<dyn subjects::IUserManager>("special")
            .await?
            .transient()?;

        Ok(())
    }

    #[tokio::test]
    async fn can_get_singleton() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &Arc<AsyncDIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;

                fn do_clone(&self) -> Box<dyn IAsyncProvider>;
            }
        }

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = ThreadsafeSingletonPtr::new(subjects::Number::new());

        ThreadsafeSingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider.expect_do_clone().returning(move || {
            let mut inner_mock_provider = MockProvider::new();

            let singleton_clone = singleton.clone();

            inner_mock_provider.expect_provide().returning(move |_, _| {
                Ok(AsyncProvidable::Singleton(singleton_clone.clone()))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<dyn subjects::INumber>(None, Box::new(mock_provider));
        }

        let first_number_rc = di_container
            .get::<dyn subjects::INumber>()
            .await?
            .threadsafe_singleton()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get::<dyn subjects::INumber>()
            .await?
            .threadsafe_singleton()?;

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());

        Ok(())
    }

    #[tokio::test]
    async fn can_get_singleton_named() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &Arc<AsyncDIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;

                fn do_clone(&self) -> Box<dyn IAsyncProvider>;
            }
        }

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = ThreadsafeSingletonPtr::new(subjects::Number::new());

        ThreadsafeSingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider.expect_do_clone().returning(move || {
            let mut inner_mock_provider = MockProvider::new();

            let singleton_clone = singleton.clone();

            inner_mock_provider.expect_provide().returning(move |_, _| {
                Ok(AsyncProvidable::Singleton(singleton_clone.clone()))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<dyn subjects::INumber>(Some("cool"), Box::new(mock_provider));
        }

        let first_number_rc = di_container
            .get_named::<dyn subjects::INumber>("cool")
            .await?
            .threadsafe_singleton()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get_named::<dyn subjects::INumber>("cool")
            .await?
            .threadsafe_singleton()?;

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_get_factory() -> Result<(), Box<dyn Error>>
    {
        trait IUserManager: Send + Sync
        {
            fn add_user(&mut self, user_id: i128);

            fn remove_user(&mut self, user_id: i128);
        }

        struct UserManager
        {
            users: Vec<i128>,
        }

        impl UserManager
        {
            fn new(users: Vec<i128>) -> Self
            {
                Self { users }
            }
        }

        impl IUserManager for UserManager
        {
            fn add_user(&mut self, user_id: i128)
            {
                self.users.push(user_id);
            }

            fn remove_user(&mut self, user_id: i128)
            {
                let user_index =
                    self.users.iter().position(|user| *user == user_id).unwrap();

                self.users.remove(user_index);
            }
        }

        use crate as syrette;

        #[crate::factory(threadsafe = true)]
        type IUserManagerFactory = dyn Fn(Vec<i128>) -> dyn IUserManager;

        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &Arc<AsyncDIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;

                fn do_clone(&self) -> Box<dyn IAsyncProvider>;
            }
        }

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            type FactoryFunc = Box<
                (dyn Fn<(Vec<i128>,), Output = TransientPtr<dyn IUserManager>> + Send + Sync)
            >;

            let mut inner_mock_provider = MockProvider::new();

            let factory_func: &'static (dyn Fn<
                (Arc<AsyncDIContainer>,),
                Output = FactoryFunc> + Send + Sync) = &|_| {
                Box::new(|users| {
                    let user_manager: TransientPtr<dyn IUserManager> =
                        TransientPtr::new(UserManager::new(users));

                    user_manager
                })
            };

            inner_mock_provider.expect_provide().returning(|_, _| {
                Ok(AsyncProvidable::Factory(
                    crate::ptr::ThreadsafeFactoryPtr::new(
                        ThreadsafeCastableFactory::new(factory_func),
                    ),
                ))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<IUserManagerFactory>(None, Box::new(mock_provider));
        }

        di_container
            .get::<IUserManagerFactory>()
            .await?
            .threadsafe_factory()?;

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_get_factory_named() -> Result<(), Box<dyn Error>>
    {
        trait IUserManager: Send + Sync
        {
            fn add_user(&mut self, user_id: i128);

            fn remove_user(&mut self, user_id: i128);
        }

        struct UserManager
        {
            users: Vec<i128>,
        }

        impl UserManager
        {
            fn new(users: Vec<i128>) -> Self
            {
                Self { users }
            }
        }

        impl IUserManager for UserManager
        {
            fn add_user(&mut self, user_id: i128)
            {
                self.users.push(user_id);
            }

            fn remove_user(&mut self, user_id: i128)
            {
                let user_index =
                    self.users.iter().position(|user| *user == user_id).unwrap();

                self.users.remove(user_index);
            }
        }

        use crate as syrette;

        #[crate::factory(threadsafe = true)]
        type IUserManagerFactory = dyn Fn(Vec<i128>) -> dyn IUserManager;

        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &Arc<AsyncDIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;

                fn do_clone(&self) -> Box<dyn IAsyncProvider>;
            }
        }

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            type FactoryFunc = Box<
                (dyn Fn<(Vec<i128>,), Output = TransientPtr<dyn IUserManager>> + Send + Sync)
            >;

            let mut inner_mock_provider = MockProvider::new();

            let factory_func: &'static (dyn Fn<
                (Arc<AsyncDIContainer>,),
                Output = FactoryFunc> + Send + Sync) = &|_| {
                Box::new(|users| {
                    let user_manager: TransientPtr<dyn IUserManager> =
                        TransientPtr::new(UserManager::new(users));

                    user_manager
                })
            };

            inner_mock_provider.expect_provide().returning(|_, _| {
                Ok(AsyncProvidable::Factory(
                    crate::ptr::ThreadsafeFactoryPtr::new(
                        ThreadsafeCastableFactory::new(factory_func),
                    ),
                ))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<IUserManagerFactory>(Some("special"), Box::new(mock_provider));
        }

        di_container
            .get_named::<IUserManagerFactory>("special")
            .await?
            .threadsafe_factory()?;

        Ok(())
    }
}
