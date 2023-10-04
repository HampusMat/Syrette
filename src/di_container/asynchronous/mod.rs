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
use std::any::type_name;

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

/// Async dependency injection container.
#[derive(Default)]
pub struct AsyncDIContainer
{
    binding_storage: DIContainerBindingStorage<dyn IAsyncProvider<Self>>,
}

impl AsyncDIContainer
{
    /// Returns a new `AsyncDIContainer`.
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            binding_storage: DIContainerBindingStorage::new(),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
impl AsyncDIContainer
{
    /// Returns a new [`AsyncBindingBuilder`] for the given interface.
    ///
    /// # Examples
    /// ```
    /// # use syrette::{AsyncDIContainer, injectable};
    /// #
    /// # struct DiskWriter {}
    /// #
    /// # #[injectable(async = true)]
    /// # impl DiskWriter
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = AsyncDIContainer::new();
    ///
    /// di_container.bind::<DiskWriter>().to::<DiskWriter>()?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn bind<Interface>(&mut self) -> AsyncBindingBuilder<'_, Interface>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        #[cfg(test)]
        panic!("Bind function is unusable when testing");

        #[cfg(not(test))]
        AsyncBindingBuilder::new(self, DependencyHistory::new)
    }

    /// Returns the type bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    ///
    /// # Examples
    /// ```
    /// # use syrette::{AsyncDIContainer, injectable};
    /// #
    /// # struct DeviceManager {}
    /// #
    /// # #[injectable(async = true)]
    /// # impl DeviceManager
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = AsyncDIContainer::new();
    ///
    /// di_container.bind::<DeviceManager>().to::<DeviceManager>()?;
    ///
    /// let device_manager = di_container.get::<DeviceManager>().await?.transient();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get<Interface>(
        &self,
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
    ///
    /// # Examples
    /// ```
    /// # use syrette::{AsyncDIContainer, injectable};
    /// #
    /// # struct DeviceManager {}
    /// #
    /// # #[injectable(async = true)]
    /// # impl DeviceManager
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = AsyncDIContainer::new();
    ///
    /// di_container
    ///     .bind::<DeviceManager>()
    ///     .to::<DeviceManager>()?
    ///     .in_transient_scope()
    ///     .when_named("usb");
    ///
    /// let device_manager = di_container
    ///     .get_named::<DeviceManager>("usb")
    ///     .await?
    ///     .transient();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_named<Interface>(
        &self,
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
        &self,
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

    fn has_binding<Interface>(&self, binding_options: BindingOptions<'static>) -> bool
    where
        Interface: ?Sized + 'static,
    {
        self.binding_storage.has::<Interface>(binding_options)
    }

    fn set_binding<Interface>(
        &mut self,
        binding_options: BindingOptions<'static>,
        provider: Box<dyn IAsyncProvider<Self>>,
    ) where
        Interface: 'static + ?Sized,
    {
        self.binding_storage
            .set::<Interface>(binding_options, provider);
    }

    fn remove_binding<Interface>(
        &mut self,
        binding_options: BindingOptions<'static>,
    ) -> Option<Box<dyn IAsyncProvider<Self>>>
    where
        Interface: 'static + ?Sized,
    {
        self.binding_storage.remove::<Interface>(binding_options)
    }
}

impl AsyncDIContainer
{
    async fn handle_binding_providable<Interface>(
        &self,
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
                    .cast::<dyn IThreadsafeFactory<Interface, Self>>()
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

                Ok(SomePtr::ThreadsafeFactory(factory.call(self).into()))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::DefaultFactory(binding) => {
                use crate::private::factory::IThreadsafeFactory;
                use crate::ptr::TransientPtr;

                type DefaultFactoryFn<Interface> = dyn IThreadsafeFactory<
                    dyn Fn<(), Output = TransientPtr<Interface>> + Send + Sync,
                    AsyncDIContainer,
                >;

                let default_factory = Self::cast_factory_binding::<
                    DefaultFactoryFn<Interface>,
                >(binding, "default factory")?;

                Ok(SomePtr::Transient(default_factory.call(self)()))
            }
            #[cfg(feature = "factory")]
            AsyncProvidable::AsyncDefaultFactory(binding) => {
                use crate::future::BoxFuture;
                use crate::private::factory::IThreadsafeFactory;
                use crate::ptr::TransientPtr;

                type AsyncDefaultFactoryFn<Interface> = dyn IThreadsafeFactory<
                    dyn Fn<(), Output = BoxFuture<'static, TransientPtr<Interface>>>
                        + Send
                        + Sync,
                    AsyncDIContainer,
                >;

                let async_default_factory = Self::cast_factory_binding::<
                    AsyncDefaultFactoryFn<Interface>,
                >(
                    binding, "async default factory"
                )?;

                Ok(SomePtr::Transient(async_default_factory.call(self)().await))
            }
        }
    }

    #[cfg(feature = "factory")]
    fn cast_factory_binding<Type: 'static + ?Sized>(
        factory_binding: std::sync::Arc<
            dyn crate::private::any_factory::AnyThreadsafeFactory,
        >,
        binding_kind: &'static str,
    ) -> Result<std::sync::Arc<Type>, AsyncDIContainerError>
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
        &self,
        binding_options: BindingOptions<'static>,
        dependency_history: DependencyHistory,
    ) -> Result<AsyncProvidable<Self>, AsyncDIContainerError>
    where
        Interface: 'static + ?Sized + Send + Sync,
    {
        let provider = self
            .binding_storage
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
    use super::*;
    use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};
    use crate::test_utils::mocks::async_provider::MockAsyncProvider;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn can_get()
    {
        let mut di_container = AsyncDIContainer::new();

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

        di_container
            .binding_storage
            .set::<dyn subjects_async::IUserManager>(
                BindingOptions::new(),
                Box::new(mock_provider),
            );

        di_container
            .get::<dyn subjects_async::IUserManager>()
            .await
            .unwrap()
            .transient()
            .unwrap();
    }

    #[tokio::test]
    async fn can_get_named()
    {
        let mut di_container = AsyncDIContainer::new();

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

        di_container
            .binding_storage
            .set::<dyn subjects_async::IUserManager>(
                BindingOptions::new().name("special"),
                Box::new(mock_provider),
            );

        di_container
            .get_named::<dyn subjects_async::IUserManager>("special")
            .await
            .unwrap()
            .transient()
            .unwrap();
    }

    #[tokio::test]
    async fn can_get_singleton()
    {
        let mut di_container = AsyncDIContainer::new();

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

        di_container
            .binding_storage
            .set::<dyn subjects_async::INumber>(
                BindingOptions::new(),
                Box::new(mock_provider),
            );

        let first_number_rc = di_container
            .get::<dyn subjects_async::INumber>()
            .await
            .unwrap()
            .threadsafe_singleton()
            .unwrap();

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get::<dyn subjects_async::INumber>()
            .await
            .unwrap()
            .threadsafe_singleton()
            .unwrap();

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());
    }

    #[tokio::test]
    async fn can_get_singleton_named()
    {
        let mut di_container = AsyncDIContainer::new();

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

        di_container
            .binding_storage
            .set::<dyn subjects_async::INumber>(
                BindingOptions::new().name("cool"),
                Box::new(mock_provider),
            );

        let first_number_rc = di_container
            .get_named::<dyn subjects_async::INumber>("cool")
            .await
            .unwrap()
            .threadsafe_singleton()
            .unwrap();

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get_named::<dyn subjects_async::INumber>("cool")
            .await
            .unwrap()
            .threadsafe_singleton()
            .unwrap();

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_get_factory()
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

        let mut di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            let mut inner_mock_provider = MockAsyncProvider::new();

            let factory_func = &|_: &AsyncDIContainer| {
                Box::new(|users| {
                    TransientPtr::new(UserManager::new(users))
                        as TransientPtr<dyn IUserManager>
                }) as Box<IUserManagerFactory>
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

        di_container
            .binding_storage
            .set::<IUserManagerFactory>(BindingOptions::new(), Box::new(mock_provider));

        di_container
            .get::<IUserManagerFactory>()
            .await
            .unwrap()
            .threadsafe_factory()
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn can_get_factory_named()
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

        let mut di_container = AsyncDIContainer::new();

        let mut mock_provider = MockAsyncProvider::new();

        mock_provider.expect_do_clone().returning(|| {
            let mut inner_mock_provider = MockAsyncProvider::new();

            let factory_func = &|_: &AsyncDIContainer| {
                Box::new(|users| {
                    TransientPtr::new(UserManager::new(users))
                        as TransientPtr<dyn IUserManager>
                }) as Box<IUserManagerFactory>
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

        di_container.binding_storage.set::<IUserManagerFactory>(
            BindingOptions::new().name("special"),
            Box::new(mock_provider),
        );

        di_container
            .get_named::<IUserManagerFactory>("special")
            .await
            .unwrap()
            .threadsafe_factory()
            .unwrap();
    }

    #[tokio::test]
    async fn has_binding_works()
    {
        let mut di_container = AsyncDIContainer::new();

        // No binding is present yet
        assert!(
            !di_container.has_binding::<subjects_async::Number>(BindingOptions::new())
        );

        di_container.binding_storage.set::<subjects_async::Number>(
            BindingOptions::new(),
            Box::new(MockAsyncProvider::new()),
        );

        assert!(di_container.has_binding::<subjects_async::Number>(BindingOptions::new()));
    }

    #[tokio::test]
    async fn set_binding_works()
    {
        let mut di_container = AsyncDIContainer::new();

        di_container.set_binding::<subjects_async::UserManager>(
            BindingOptions::new(),
            Box::new(MockAsyncProvider::new()),
        );

        assert!(di_container
            .binding_storage
            .has::<subjects_async::UserManager>(BindingOptions::new()));
    }

    #[tokio::test]
    async fn remove_binding_works()
    {
        let mut di_container = AsyncDIContainer::new();

        di_container
            .binding_storage
            .set::<subjects_async::UserManager>(
                BindingOptions::new(),
                Box::new(MockAsyncProvider::new()),
            );

        assert!(
            // Formatting is weird without this comment
            di_container
                .remove_binding::<subjects_async::UserManager>(BindingOptions::new())
                .is_some()
        );

        assert!(
            // Formatting is weird without this comment
            !di_container
                .binding_storage
                .has::<subjects_async::UserManager>(BindingOptions::new())
        );
    }
}
