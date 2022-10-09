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
use std::any::type_name;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::di_container::asynchronous::binding::builder::AsyncBindingBuilder;
use crate::di_container::binding_map::DIContainerBindingMap;
use crate::errors::async_di_container::AsyncDIContainerError;
use crate::future::BoxFuture;
use crate::libs::intertrait::cast::error::CastError;
use crate::libs::intertrait::cast::{CastArc, CastBox};
use crate::provider::r#async::{AsyncProvidable, IAsyncProvider};
use crate::ptr::{SomeThreadsafePtr, TransientPtr};

pub mod binding;

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
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
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
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
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
    use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};
    use crate::test_utils::subjects_async;

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
                    subjects_async::UserManager::new(),
                )))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<dyn subjects_async::IUserManager>(None, Box::new(mock_provider));
        }

        di_container
            .get::<dyn subjects_async::IUserManager>()
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
                    subjects_async::UserManager::new(),
                )))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .bindings
                .lock()
                .await
                .set::<dyn subjects_async::IUserManager>(
                    Some("special"),
                    Box::new(mock_provider),
                );
        }

        di_container
            .get_named::<dyn subjects_async::IUserManager>("special")
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

        let mut singleton = ThreadsafeSingletonPtr::new(subjects_async::Number::new());

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
                .set::<dyn subjects_async::INumber>(None, Box::new(mock_provider));
        }

        let first_number_rc = di_container
            .get::<dyn subjects_async::INumber>()
            .await?
            .threadsafe_singleton()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get::<dyn subjects_async::INumber>()
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

        let mut singleton = ThreadsafeSingletonPtr::new(subjects_async::Number::new());

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
                .set::<dyn subjects_async::INumber>(
                    Some("cool"),
                    Box::new(mock_provider),
                );
        }

        let first_number_rc = di_container
            .get_named::<dyn subjects_async::INumber>("cool")
            .await?
            .threadsafe_singleton()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get_named::<dyn subjects_async::INumber>("cool")
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
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;

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
        use crate::castable_factory::threadsafe::ThreadsafeCastableFactory;

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
