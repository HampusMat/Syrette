//! Asynchronous dependency injection container.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use std::error::Error;
//!
//! use syrette::{injectable, AsyncDIContainer};
//!
//! trait IDatabaseService
//! {
//!     fn get_all_records(&self, table_name: String) -> HashMap<String, String>;
//! }
//!
//! struct DatabaseService {}
//!
//! #[injectable(IDatabaseService, { async = true })]
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
//!         .to::<DatabaseService>()?;
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

#[cfg(feature = "factory")]
use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;
use crate::di_container_binding_map::DIContainerBindingMap;
use crate::errors::async_di_container::{
    AsyncBindingBuilderError,
    AsyncBindingScopeConfiguratorError,
    AsyncBindingWhenConfiguratorError,
    AsyncDIContainerError,
};
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::libs::intertrait::cast::error::CastError;
use crate::libs::intertrait::cast::{CastArc, CastBox};
use crate::provider::r#async::{
    AsyncProvidable,
    AsyncSingletonProvider,
    AsyncTransientTypeProvider,
    IAsyncProvider,
};
use crate::ptr::{SomeThreadsafePtr, ThreadsafeSingletonPtr};

/// When configurator for a binding for type 'Interface' inside a [`AsyncDIContainer`].
pub struct AsyncBindingWhenConfigurator<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: &'di_container mut AsyncDIContainer,
    interface_phantom: PhantomData<Interface>,
}

impl<'di_container, Interface> AsyncBindingWhenConfigurator<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    fn new(di_container: &'di_container mut AsyncDIContainer) -> Self
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
    pub fn when_named(
        &mut self,
        name: &'static str,
    ) -> Result<(), AsyncBindingWhenConfiguratorError>
    {
        let binding = self
            .di_container
            .bindings
            .remove::<Interface>(None)
            .map_or_else(
                || {
                    Err(AsyncBindingWhenConfiguratorError::BindingNotFound(
                        type_name::<Interface>(),
                    ))
                },
                Ok,
            )?;

        self.di_container
            .bindings
            .set::<Interface>(Some(name), binding);

        Ok(())
    }
}

/// Scope configurator for a binding for type 'Interface' inside a [`AsyncDIContainer`].
pub struct AsyncBindingScopeConfigurator<'di_container, Interface, Implementation>
where
    Interface: 'static + ?Sized,
    Implementation: AsyncInjectable,
{
    di_container: &'di_container mut AsyncDIContainer,
    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<'di_container, Interface, Implementation>
    AsyncBindingScopeConfigurator<'di_container, Interface, Implementation>
where
    Interface: 'static + ?Sized,
    Implementation: AsyncInjectable,
{
    fn new(di_container: &'di_container mut AsyncDIContainer) -> Self
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
    pub fn in_transient_scope(&mut self) -> AsyncBindingWhenConfigurator<Interface>
    {
        self.di_container.bindings.set::<Interface>(
            None,
            Box::new(AsyncTransientTypeProvider::<Implementation>::new()),
        );

        AsyncBindingWhenConfigurator::new(self.di_container)
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub async fn in_singleton_scope(
        &mut self,
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingScopeConfiguratorError>
    {
        let singleton: ThreadsafeSingletonPtr<Implementation> =
            ThreadsafeSingletonPtr::from(
                Implementation::resolve(self.di_container, Vec::new())
                    .await
                    .map_err(
                        AsyncBindingScopeConfiguratorError::SingletonResolveFailed,
                    )?,
            );

        self.di_container
            .bindings
            .set::<Interface>(None, Box::new(AsyncSingletonProvider::new(singleton)));

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }
}

/// Binding builder for type `Interface` inside a [`AsyncDIContainer`].
pub struct AsyncBindingBuilder<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: &'di_container mut AsyncDIContainer,
    interface_phantom: PhantomData<Interface>,
}

impl<'di_container, Interface> AsyncBindingBuilder<'di_container, Interface>
where
    Interface: 'static + ?Sized,
{
    fn new(di_container: &'di_container mut AsyncDIContainer) -> Self
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
    pub fn to<Implementation>(
        &mut self,
    ) -> Result<
        AsyncBindingScopeConfigurator<Interface, Implementation>,
        AsyncBindingBuilderError,
    >
    where
        Implementation: AsyncInjectable,
    {
        if self.di_container.bindings.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let mut binding_scope_configurator =
            AsyncBindingScopeConfigurator::new(self.di_container);

        binding_scope_configurator.in_transient_scope();

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
    pub fn to_factory<Args, Return>(
        &mut self,
        factory_func: &'static (dyn Fn<Args, Output = crate::ptr::TransientPtr<Return>>
                      + Send
                      + Sync),
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: crate::interfaces::factory::IFactory<Args, Return>,
    {
        if self.di_container.bindings.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        self.di_container.bindings.set::<Interface>(
            None,
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
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
    pub fn to_default_factory<Return>(
        &mut self,
        factory_func: &'static (dyn Fn<(), Output = crate::ptr::TransientPtr<Return>>
                      + Send
                      + Sync),
    ) -> Result<AsyncBindingWhenConfigurator<Interface>, AsyncBindingBuilderError>
    where
        Return: 'static + ?Sized,
    {
        if self.di_container.bindings.has::<Interface>(None) {
            return Err(AsyncBindingBuilderError::BindingAlreadyExists(type_name::<
                Interface,
            >(
            )));
        }

        let factory_impl = ThreadsafeCastableFactory::new(factory_func);

        self.di_container.bindings.set::<Interface>(
            None,
            Box::new(crate::provider::r#async::AsyncFactoryProvider::new(
                crate::ptr::ThreadsafeFactoryPtr::new(factory_impl),
            )),
        );

        Ok(AsyncBindingWhenConfigurator::new(self.di_container))
    }
}

/// Dependency injection container.
pub struct AsyncDIContainer
{
    bindings: DIContainerBindingMap<dyn IAsyncProvider>,
}

impl AsyncDIContainer
{
    /// Returns a new `AsyncDIContainer`.
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            bindings: DIContainerBindingMap::new(),
        }
    }

    /// Returns a new [`AsyncBindingBuilder`] for the given interface.
    pub fn bind<Interface>(&mut self) -> AsyncBindingBuilder<Interface>
    where
        Interface: 'static + ?Sized,
    {
        AsyncBindingBuilder::<Interface>::new(self)
    }

    /// Returns the type bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for fails
    /// - Casting the binding for fails
    pub async fn get<Interface>(
        &self,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized,
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
        &self,
        name: &'static str,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(Vec::new(), Some(name)).await
    }

    #[doc(hidden)]
    pub async fn get_bound<Interface>(
        &self,
        dependency_history: Vec<&'static str>,
        name: Option<&'static str>,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let binding_providable = self
            .get_binding_providable::<Interface>(name, dependency_history)
            .await?;

        Self::handle_binding_providable(binding_providable)
    }

    fn handle_binding_providable<Interface>(
        binding_providable: AsyncProvidable,
    ) -> Result<SomeThreadsafePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized,
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
                match factory_binding.clone().cast::<Interface>() {
                    Ok(factory) => Ok(SomeThreadsafePtr::ThreadsafeFactory(factory)),
                    Err(first_err) => {
                        use crate::interfaces::factory::IFactory;

                        if let CastError::NotArcCastable(_) = first_err {
                            return Err(AsyncDIContainerError::InterfaceNotAsync(
                                type_name::<Interface>(),
                            ));
                        }

                        let default_factory = factory_binding
                            .cast::<dyn IFactory<(), Interface>>()
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
                                        binding_kind: "factory",
                                    }
                                }
                            })?;

                        Ok(SomeThreadsafePtr::Transient(default_factory()))
                    }
                }
            }
        }
    }

    async fn get_binding_providable<Interface>(
        &self,
        name: Option<&'static str>,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.bindings
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
            .provide(self, dependency_history)
            .await
            .map_err(|err| AsyncDIContainerError::BindingResolveFailed {
                reason: err,
                interface: type_name::<Interface>(),
            })
    }
}

impl Default for AsyncDIContainer
{
    fn default() -> Self
    {
        Self::new()
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

        use async_trait::async_trait;
        use syrette_macros::declare_interface;

        use super::AsyncDIContainer;
        use crate::interfaces::async_injectable::AsyncInjectable;
        use crate::ptr::TransientPtr;

        pub trait IUserManager
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
                _: &AsyncDIContainer,
                _dependency_history: Vec<&'static str>,
            ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
            where
                Self: Sized,
            {
                Ok(TransientPtr::new(Self::new()))
            }
        }

        pub trait INumber
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
                _: &AsyncDIContainer,
                _dependency_history: Vec<&'static str>,
            ) -> Result<TransientPtr<Self>, crate::errors::injectable::InjectableError>
            where
                Self: Sized,
            {
                Ok(TransientPtr::new(Self::new()))
            }
        }
    }

    #[test]
    fn can_bind_to() -> Result<(), Box<dyn Error>>
    {
        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_transient() -> Result<(), Box<dyn Error>>
    {
        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_transient_scope();

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_transient_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_transient_scope()
            .when_named("regular")?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn can_bind_to_singleton() -> Result<(), Box<dyn Error>>
    {
        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_singleton_scope()
            .await?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn can_bind_to_singleton_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_singleton_scope()
            .await?
            .when_named("cool")?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory() -> Result<(), Box<dyn Error>>
    {
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(), dyn subjects::IUserManager>;

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container.bind::<IUserManagerFactory>().to_factory(&|| {
            let user_manager: TransientPtr<dyn subjects::IUserManager> =
                TransientPtr::new(subjects::UserManager::new());

            user_manager
        })?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory_when_named() -> Result<(), Box<dyn Error>>
    {
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(), dyn subjects::IUserManager>;

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|| {
                let user_manager: TransientPtr<dyn subjects::IUserManager> =
                    TransientPtr::new(subjects::UserManager::new());

                user_manager
            })?
            .when_named("awesome")?;

        assert_eq!(di_container.bindings.count(), 1);

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
                    di_container: &AsyncDIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;
            }
        }

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(AsyncProvidable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .bindings
            .set::<dyn subjects::IUserManager>(None, Box::new(mock_provider));

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
                    di_container: &AsyncDIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;
            }
        }

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(AsyncProvidable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .bindings
            .set::<dyn subjects::IUserManager>(Some("special"), Box::new(mock_provider));

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
                    di_container: &AsyncDIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;
            }
        }

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = ThreadsafeSingletonPtr::new(subjects::Number::new());

        ThreadsafeSingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(AsyncProvidable::Singleton(singleton.clone())));

        di_container
            .bindings
            .set::<dyn subjects::INumber>(None, Box::new(mock_provider));

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
                    di_container: &AsyncDIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;
            }
        }

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = ThreadsafeSingletonPtr::new(subjects::Number::new());

        ThreadsafeSingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(AsyncProvidable::Singleton(singleton.clone())));

        di_container
            .bindings
            .set::<dyn subjects::INumber>(Some("cool"), Box::new(mock_provider));

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
        trait IUserManager
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
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(Vec<i128>,), dyn IUserManager>;

        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &AsyncDIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;
            }
        }

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(AsyncProvidable::Factory(
                crate::ptr::ThreadsafeFactoryPtr::new(ThreadsafeCastableFactory::new(
                    &|users| {
                        let user_manager: TransientPtr<dyn IUserManager> =
                            TransientPtr::new(UserManager::new(users));

                        user_manager
                    },
                )),
            ))
        });

        di_container
            .bindings
            .set::<IUserManagerFactory>(None, Box::new(mock_provider));

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
        trait IUserManager
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
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(Vec<i128>,), dyn IUserManager>;

        mock! {
            Provider {}

            #[async_trait]
            impl IAsyncProvider for Provider
            {
                async fn provide(
                    &self,
                    di_container: &AsyncDIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<AsyncProvidable, InjectableError>;
            }
        }

        let mut di_container: AsyncDIContainer = AsyncDIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(AsyncProvidable::Factory(
                crate::ptr::ThreadsafeFactoryPtr::new(ThreadsafeCastableFactory::new(
                    &|users| {
                        let user_manager: TransientPtr<dyn IUserManager> =
                            TransientPtr::new(UserManager::new(users));

                        user_manager
                    },
                )),
            ))
        });

        di_container
            .bindings
            .set::<IUserManagerFactory>(Some("special"), Box::new(mock_provider));

        di_container
            .get_named::<IUserManagerFactory>("special")
            .await?
            .threadsafe_factory()?;

        Ok(())
    }
}
