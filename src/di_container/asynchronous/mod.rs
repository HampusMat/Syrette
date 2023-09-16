//! Asynchronous dependency injection container.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use std::error::Error;
//!
//! use syrette::di_container::asynchronous::prelude::*;
//! use syrette::injectable;
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

use async_lock::Mutex;

use crate::di_container::asynchronous::binding::builder::AsyncBindingBuilder;
use crate::di_container::binding_storage::DIContainerBindingStorage;
use crate::di_container::BindingOptions;
use crate::errors::async_di_container::AsyncDIContainerError;
use crate::private::cast::arc::CastArc;
use crate::private::cast::boxed::CastBox;
use crate::private::cast::error::CastError;
use crate::provider::r#async::{AsyncProvidable, IAsyncProvider};
use crate::ptr::SomePtr;
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

pub mod binding;
pub mod prelude;

/// Async dependency injection container.
pub struct AsyncDIContainer
{
    binding_storage: Mutex<DIContainerBindingStorage<dyn IAsyncProvider<Self>>>,
}

impl AsyncDIContainer
{
    /// Returns a new `AsyncDIContainer`.
    #[must_use]
    pub fn new() -> Arc<Self>
    {
        Arc::new(Self {
            binding_storage: Mutex::new(DIContainerBindingStorage::new()),
        })
    }
}

#[cfg_attr(test, mockall::automock)]
impl AsyncDIContainer
{
    /// Returns a new [`AsyncBindingBuilder`] for the given interface.
    #[allow(clippy::missing_panics_doc)]
    pub fn bind<Interface>(self: &mut Arc<Self>) -> AsyncBindingBuilder<Interface>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        #[cfg(test)]
        panic!("Bind function is unusable when testing");

        #[cfg(not(test))]
        AsyncBindingBuilder::new(self.clone(), DependencyHistory::new)
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
    ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        self.get_bound::<Interface>(DependencyHistory::new(), BindingOptions::new())
            .await
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
    ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        self.get_bound::<Interface>(
            DependencyHistory::new(),
            BindingOptions::new().name(name),
        )
        .await
    }

    /// Returns the type bound with `Interface` where the binding has the specified
    /// options.
    ///
    /// `dependency_history` is passed to the bound type when it is being resolved.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    ///
    /// # Examples
    /// ```
    /// # use syrette::di_container::asynchronous::AsyncDIContainer;
    /// # use syrette::dependency_history::DependencyHistory;
    /// # use syrette::di_container::BindingOptions;
    /// #
    /// # struct EventHandler {}
    /// # struct Button {}
    /// #
    /// # Box::pin(async {
    /// # let di_container = AsyncDIContainer::new();
    /// #
    /// let mut dependency_history = DependencyHistory::new();
    ///
    /// dependency_history.push::<EventHandler>();
    ///
    /// di_container
    ///     .get_bound::<Button>(dependency_history, BindingOptions::new().name("huge"))
    ///     .await?;
    /// #
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn get_bound<Interface>(
        self: &Arc<Self>,
        dependency_history: DependencyHistory,
        binding_options: BindingOptions<'static>,
    ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        let binding_providable = self
            .get_binding_providable::<Interface>(binding_options, dependency_history)
            .await?;

        self.handle_binding_providable(binding_providable).await
    }

    async fn has_binding<Interface>(
        self: &Arc<Self>,
        binding_options: BindingOptions<'static>,
    ) -> bool
    where
        Interface: ?Sized + 'static,
    {
        self.binding_storage
            .lock()
            .await
            .has::<Interface>(binding_options)
    }

    async fn set_binding<Interface>(
        self: &Arc<Self>,
        binding_options: BindingOptions<'static>,
        provider: Box<dyn IAsyncProvider<Self>>,
    ) where
        Interface: 'static + ?Sized,
    {
        self.binding_storage
            .lock()
            .await
            .set::<Interface>(binding_options, provider);
    }

    async fn remove_binding<Interface>(
        self: &Arc<Self>,
        binding_options: BindingOptions<'static>,
    ) -> Option<Box<dyn IAsyncProvider<Self>>>
    where
        Interface: 'static + ?Sized,
    {
        self.binding_storage
            .lock()
            .await
            .remove::<Interface>(binding_options)
    }
}

impl AsyncDIContainer
{
    async fn handle_binding_providable<Interface>(
        self: &Arc<Self>,
        binding_providable: AsyncProvidable<Self>,
    ) -> Result<SomePtr<Interface>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        match binding_providable {
            AsyncProvidable::Transient(transient_binding) => Ok(SomePtr::Transient(
                transient_binding.cast::<Interface>().map_err(|_| {
                    AsyncDIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "transient",
                    }
                })?,
            )),
            AsyncProvidable::Singleton(singleton_binding) => {
                Ok(SomePtr::ThreadsafeSingleton(
                    singleton_binding
                        .cast::<Interface>()
                        .map_err(|err| match err {
                            CastError::NotArcCastable(_) => {
                                AsyncDIContainerError::InterfaceNotAsync(type_name::<
                                    Interface,
                                >(
                                ))
                            }
                            CastError::CastFailed {
                                source: _,
                                from: _,
                                to: _,
                            }
                            | CastError::GetCasterFailed(_) => {
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
                use crate::private::factory::IThreadsafeFactory;

                let factory = factory_binding
                    .cast::<dyn IThreadsafeFactory<(Arc<AsyncDIContainer>,), Interface>>()
                    .map_err(|err| match err {
                        CastError::NotArcCastable(_) => {
                            AsyncDIContainerError::InterfaceNotAsync(
                                type_name::<Interface>(),
                            )
                        }
                        CastError::CastFailed {
                            source: _,
                            from: _,
                            to: _,
                        }
                        | CastError::GetCasterFailed(_) => {
                            AsyncDIContainerError::CastFailed {
                                interface: type_name::<Interface>(),
                                binding_kind: "factory",
                            }
                        }
                    })?;

                Ok(SomePtr::ThreadsafeFactory(factory(self.clone()).into()))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::DefaultFactory(binding) => {
                use crate::private::factory::IThreadsafeFactory;
                use crate::ptr::TransientPtr;

                let default_factory = Self::cast_factory_binding::<
                    dyn IThreadsafeFactory<
                        (Arc<AsyncDIContainer>,),
                        dyn Fn<(), Output = TransientPtr<Interface>> + Send + Sync,
                    >,
                >(binding, "default factory")?;

                Ok(SomePtr::Transient(default_factory(self.clone())()))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::AsyncDefaultFactory(binding) => {
                use crate::future::BoxFuture;
                use crate::private::factory::IThreadsafeFactory;
                use crate::ptr::TransientPtr;

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

                Ok(SomePtr::Transient(
                    async_default_factory(self.clone())().await,
                ))
            }
        }
    }

    #[cfg(feature = "factory")]
    fn cast_factory_binding<Type: 'static + ?Sized>(
        factory_binding: Arc<dyn crate::private::any_factory::AnyThreadsafeFactory>,
        binding_kind: &'static str,
    ) -> Result<Arc<Type>, AsyncDIContainerError>
    {
        factory_binding.cast::<Type>().map_err(|err| match err {
            CastError::NotArcCastable(_) => {
                AsyncDIContainerError::InterfaceNotAsync(type_name::<Type>())
            }
            CastError::CastFailed {
                source: _,
                from: _,
                to: _,
            }
            | CastError::GetCasterFailed(_) => AsyncDIContainerError::CastFailed {
                interface: type_name::<Type>(),
                binding_kind,
            },
        })
    }

    async fn get_binding_providable<Interface>(
        self: &Arc<Self>,
        binding_options: BindingOptions<'static>,
        dependency_history: DependencyHistory,
    ) -> Result<AsyncProvidable<Self>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        let provider;

        {
            let bindings_lock = self.binding_storage.lock().await;

            provider = bindings_lock
                .get::<Interface>(binding_options.clone())
                .map_or_else(
                    || {
                        Err(AsyncDIContainerError::BindingNotFound {
                            interface: type_name::<Interface>(),
                            name: binding_options.name,
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

    use super::*;
    use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};
    use crate::test_utils::mocks::async_provider::MockAsyncProvider;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn can_get() -> Result<(), Box<dyn Error>>
    {
        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            let mut inner_mock_provider = MockAsyncProvider::new();

            inner_mock_provider.expect_provide().returning(|_, _| {
                Ok(AsyncProvidable::Transient(TransientPtr::new(
                    subjects_async::UserManager::new(),
                )))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .binding_storage
                .lock()
                .await
                .set::<dyn subjects_async::IUserManager>(
                    BindingOptions::new(),
                    Box::new(mock_provider),
                );
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
        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            let mut inner_mock_provider = MockAsyncProvider::new();

            inner_mock_provider.expect_provide().returning(|_, _| {
                Ok(AsyncProvidable::Transient(TransientPtr::new(
                    subjects_async::UserManager::new(),
                )))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .binding_storage
                .lock()
                .await
                .set::<dyn subjects_async::IUserManager>(
                    BindingOptions::new().name("special"),
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
        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        let mut singleton = ThreadsafeSingletonPtr::new(subjects_async::Number::new());

        ThreadsafeSingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider.expect_do_clone().returning(move || {
            let mut inner_mock_provider = MockAsyncProvider::new();

            let singleton_clone = singleton.clone();

            inner_mock_provider.expect_provide().returning(move |_, _| {
                Ok(AsyncProvidable::Singleton(singleton_clone.clone()))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .binding_storage
                .lock()
                .await
                .set::<dyn subjects_async::INumber>(
                    BindingOptions::new(),
                    Box::new(mock_provider),
                );
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
        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        let mut singleton = ThreadsafeSingletonPtr::new(subjects_async::Number::new());

        ThreadsafeSingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider.expect_do_clone().returning(move || {
            let mut inner_mock_provider = MockAsyncProvider::new();

            let singleton_clone = singleton.clone();

            inner_mock_provider.expect_provide().returning(move |_, _| {
                Ok(AsyncProvidable::Singleton(singleton_clone.clone()))
            });

            Box::new(inner_mock_provider)
        });

        {
            di_container
                .binding_storage
                .lock()
                .await
                .set::<dyn subjects_async::INumber>(
                    BindingOptions::new().name("cool"),
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
        use crate::private::castable_factory::threadsafe::ThreadsafeCastableFactory;

        #[crate::factory(threadsafe = true)]
        type IUserManagerFactory =
            dyn Fn(Vec<i128>) -> TransientPtr<dyn IUserManager> + Send + Sync;

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            type FactoryFunc = Box<
                (dyn Fn<(Vec<i128>,), Output = TransientPtr<dyn IUserManager>> + Send + Sync)
            >;

            let mut inner_mock_provider = MockAsyncProvider::new();

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
                .binding_storage
                .lock()
                .await
                .set::<IUserManagerFactory>(
                    BindingOptions::new(),
                    Box::new(mock_provider),
                );
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
        use crate::private::castable_factory::threadsafe::ThreadsafeCastableFactory;

        #[crate::factory(threadsafe = true)]
        type IUserManagerFactory =
            dyn Fn(Vec<i128>) -> TransientPtr<dyn IUserManager> + Send + Sync;

        let di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            type FactoryFunc = Box<
                (dyn Fn<(Vec<i128>,), Output = TransientPtr<dyn IUserManager>> + Send + Sync)
            >;

            let mut inner_mock_provider = MockAsyncProvider::new();

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
                .binding_storage
                .lock()
                .await
                .set::<IUserManagerFactory>(
                    BindingOptions::new().name("special"),
                    Box::new(mock_provider),
                );
        }

        di_container
            .get_named::<IUserManagerFactory>("special")
            .await?
            .threadsafe_factory()?;

        Ok(())
    }

    #[tokio::test]
    async fn has_binding_works()
    {
        let di_container = AsyncDIContainer::new();

        // No binding is present yet
        assert!(
            !di_container
                .has_binding::<subjects_async::Number>(BindingOptions::new())
                .await
        );

        di_container
            .binding_storage
            .lock()
            .await
            .set::<subjects_async::Number>(
                BindingOptions::new(),
                Box::new(MockAsyncProvider::new()),
            );

        assert!(
            di_container
                .has_binding::<subjects_async::Number>(BindingOptions::new())
                .await
        );
    }

    #[tokio::test]
    async fn set_binding_works()
    {
        let di_container = AsyncDIContainer::new();

        di_container
            .set_binding::<subjects_async::UserManager>(
                BindingOptions::new(),
                Box::new(MockAsyncProvider::new()),
            )
            .await;

        assert!(di_container
            .binding_storage
            .lock()
            .await
            .has::<subjects_async::UserManager>(BindingOptions::new()));
    }

    #[tokio::test]
    async fn remove_binding_works()
    {
        let di_container = AsyncDIContainer::new();

        di_container
            .binding_storage
            .lock()
            .await
            .set::<subjects_async::UserManager>(
                BindingOptions::new(),
                Box::new(MockAsyncProvider::new()),
            );

        assert!(
            // Formatting is weird without this comment
            di_container
                .remove_binding::<subjects_async::UserManager>(BindingOptions::new())
                .await
                .is_some()
        );

        assert!(
            // Formatting is weird without this comment
            !di_container
                .binding_storage
                .lock()
                .await
                .has::<subjects_async::UserManager>(BindingOptions::new())
        );
    }
}
