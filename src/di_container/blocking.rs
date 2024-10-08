//! Blocking dependency injection container.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use std::error::Error;
//!
//! use syrette::{injectable, DIContainer};
//!
//! trait IDatabaseService
//! {
//!     fn get_all_records(&self, table_name: String) -> HashMap<String, String>;
//! }
//!
//! struct DatabaseService {}
//!
//! #[injectable(IDatabaseService)]
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
//! fn main() -> Result<(), Box<dyn Error>>
//! {
//!     let mut di_container = DIContainer::new();
//!
//!     di_container
//!         .bind::<dyn IDatabaseService>()
//!         .to::<DatabaseService>()
//!         .map_err(|err| err.to_string())?;
//!
//!     let database_service = di_container
//!         .get::<dyn IDatabaseService>()
//!         .map_err(|err| err.to_string())?
//!         .transient()?;
//!
//!     Ok(())
//! }
//! ```
use std::any::type_name;

use crate::castable_function::CastableFunction;
use crate::di_container::binding_storage::DIContainerBindingStorage;
use crate::di_container::blocking::binding::builder::BindingBuilder;
use crate::di_container::BindingOptions;
use crate::errors::di_container::DIContainerError;
use crate::private::cast::boxed::CastBox;
use crate::private::cast::rc::CastRc;
use crate::provider::blocking::{IProvider, Providable, ProvidableFunctionKind};
use crate::ptr::{SomePtr, TransientPtr};
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

pub mod binding;

#[cfg(not(test))]
pub(crate) type BindingOptionsWithLt<'a> = BindingOptions<'a>;

#[cfg(test)]
pub(crate) type BindingOptionsWithLt = BindingOptions<'static>;

/// Blocking dependency injection container.
#[derive(Default)]
pub struct DIContainer
{
    binding_storage: DIContainerBindingStorage<dyn IProvider<Self>>,
}

impl DIContainer
{
    /// Returns a new `DIContainer`.
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            binding_storage: DIContainerBindingStorage::new(),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
impl DIContainer
{
    /// Returns a new [`BindingBuilder`] for the given interface.
    ///
    /// # Examples
    /// ```
    /// # use syrette::{DIContainer, injectable};
    /// #
    /// # struct DiskWriter {}
    /// #
    /// # #[injectable]
    /// # impl DiskWriter
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = DIContainer::new();
    ///
    /// di_container.bind::<DiskWriter>().to::<DiskWriter>()?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn bind<Interface>(&mut self) -> BindingBuilder<'_, Interface>
    where
        Interface: 'static + ?Sized,
    {
        #[cfg(test)]
        panic!("Nope");

        #[cfg(not(test))]
        BindingBuilder::new(self, DependencyHistory::new)
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
    /// # use syrette::{DIContainer, injectable};
    /// #
    /// # struct DeviceManager {}
    /// #
    /// # #[injectable]
    /// # impl DeviceManager
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = DIContainer::new();
    ///
    /// di_container.bind::<DeviceManager>().to::<DeviceManager>()?;
    ///
    /// let device_manager = di_container.get::<DeviceManager>()?.transient();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<Interface>(&self) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(DependencyHistory::new(), BindingOptions::new())
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
    /// # use syrette::{DIContainer, injectable};
    /// #
    /// # struct DeviceManager {}
    /// #
    /// # #[injectable]
    /// # impl DeviceManager
    /// # {
    /// #     fn new() -> Self
    /// #     {
    /// #         Self {}
    /// #     }
    /// # }
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut di_container = DIContainer::new();
    ///
    /// di_container
    ///     .bind::<DeviceManager>()
    ///     .to::<DeviceManager>()?
    ///     .in_transient_scope()
    ///     .when_named("usb")?;
    ///
    /// let device_manager = di_container.get_named::<DeviceManager>("usb")?.transient();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_named<Interface>(
        &self,
        name: &'static str,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(
            DependencyHistory::new(),
            BindingOptions::new().name(name),
        )
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
    /// ```no_run
    /// # use syrette::di_container::blocking::DIContainer;
    /// # use syrette::dependency_history::DependencyHistory;
    /// # use syrette::di_container::BindingOptions;
    /// #
    /// # struct EventHandler {}
    /// # struct Button {}
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let di_container = DIContainer::new();
    /// #
    /// let mut dependency_history = DependencyHistory::new();
    ///
    /// dependency_history.push::<EventHandler>();
    ///
    /// di_container.get_bound::<Button>(
    ///     dependency_history,
    ///     BindingOptions::new().name("huge_red"),
    /// )?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_bound<Interface>(
        &self,
        dependency_history: DependencyHistory,
        binding_options: BindingOptionsWithLt,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let binding_providable = self
            .get_binding_providable::<Interface>(binding_options, dependency_history)?;

        match binding_providable {
            Providable::Transient(transient_binding) => Ok(SomePtr::Transient(
                transient_binding.cast::<Interface>().map_err(|_| {
                    DIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "transient",
                    }
                })?,
            )),
            Providable::Singleton(singleton_binding) => Ok(SomePtr::Singleton(
                singleton_binding.cast::<Interface>().map_err(|_| {
                    DIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "singleton",
                    }
                })?,
            )),
            #[cfg(feature = "factory")]
            Providable::Function(func_bound, ProvidableFunctionKind::UserCalled) => {
                let factory = func_bound
                    .as_any()
                    .downcast_ref::<CastableFunction<Interface, Self>>()
                    .ok_or_else(|| DIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "factory",
                    })?;

                Ok(SomePtr::Factory(factory.call(self).into()))
            }
            Providable::Function(func_bound, ProvidableFunctionKind::Instant) => {
                type Func<Interface> =
                    CastableFunction<dyn Fn() -> TransientPtr<Interface>, DIContainer>;

                let dynamic_val_func = func_bound
                    .as_any()
                    .downcast_ref::<Func<Interface>>()
                    .ok_or_else(|| DIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "dynamic value function",
                    })?;

                Ok(SomePtr::Transient(dynamic_val_func.call(self)()))
            }
        }
    }

    fn has_binding<Interface>(&self, binding_options: BindingOptionsWithLt) -> bool
    where
        Interface: ?Sized + 'static,
    {
        self.binding_storage.has::<Interface>(binding_options)
    }

    fn set_binding<Interface>(
        &mut self,
        binding_options: BindingOptions<'static>,
        provider: Box<dyn IProvider<Self>>,
    ) where
        Interface: 'static + ?Sized,
    {
        self.binding_storage
            .set::<Interface>(binding_options, provider);
    }

    fn remove_binding<Interface>(
        &mut self,
        binding_options: BindingOptions<'static>,
    ) -> Option<Box<dyn IProvider<Self>>>
    where
        Interface: 'static + ?Sized,
    {
        self.binding_storage.remove::<Interface>(binding_options)
    }
}

impl DIContainer
{
    fn get_binding_providable<Interface>(
        &self,
        binding_options: BindingOptionsWithLt,
        dependency_history: DependencyHistory,
    ) -> Result<Providable<Self>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let name = binding_options.name;

        self.binding_storage
            .get::<Interface>(binding_options)
            .map_or_else(
                || {
                    Err(DIContainerError::BindingNotFound {
                        interface: type_name::<Interface>(),
                        name: name.as_ref().map(ToString::to_string),
                    })
                },
                Ok,
            )?
            .provide(self, dependency_history)
            .map_err(|err| DIContainerError::BindingResolveFailed {
                reason: err,
                interface: type_name::<Interface>(),
            })
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::provider::blocking::MockIProvider;
    use crate::ptr::{SingletonPtr, TransientPtr};
    use crate::test_utils::subjects;

    #[test]
    fn can_get()
    {
        let mut di_container = DIContainer::new();

        let mut mock_provider = MockIProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(Providable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .binding_storage
            .set::<dyn subjects::IUserManager>(
                BindingOptions::new(),
                Box::new(mock_provider),
            );

        di_container
            .get::<dyn subjects::IUserManager>()
            .unwrap()
            .transient()
            .unwrap();
    }

    #[test]
    fn can_get_named()
    {
        let mut di_container = DIContainer::new();

        let mut mock_provider = MockIProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(Providable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .binding_storage
            .set::<dyn subjects::IUserManager>(
                BindingOptions::new().name("special"),
                Box::new(mock_provider),
            );

        di_container
            .get_named::<dyn subjects::IUserManager>("special")
            .unwrap()
            .transient()
            .unwrap();
    }

    #[test]
    fn can_get_singleton()
    {
        let mut di_container = DIContainer::new();

        let mut mock_provider = MockIProvider::new();

        let mut singleton = SingletonPtr::new(subjects::Number::new());

        SingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(Providable::Singleton(singleton.clone())));

        di_container
            .binding_storage
            .set::<dyn subjects::INumber>(BindingOptions::new(), Box::new(mock_provider));

        let first_number_rc = di_container
            .get::<dyn subjects::INumber>()
            .unwrap()
            .singleton()
            .unwrap();

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get::<dyn subjects::INumber>()
            .unwrap()
            .singleton()
            .unwrap();

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());
    }

    #[test]
    fn can_get_singleton_named()
    {
        let mut di_container = DIContainer::new();

        let mut mock_provider = MockIProvider::new();

        let mut singleton = SingletonPtr::new(subjects::Number::new());

        SingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(Providable::Singleton(singleton.clone())));

        di_container.binding_storage.set::<dyn subjects::INumber>(
            BindingOptions::new().name("cool"),
            Box::new(mock_provider),
        );

        let first_number_rc = di_container
            .get_named::<dyn subjects::INumber>("cool")
            .unwrap()
            .singleton()
            .unwrap();

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get_named::<dyn subjects::INumber>("cool")
            .unwrap()
            .singleton()
            .unwrap();

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory()
    {
        use std::rc::Rc;

        use crate::castable_function::CastableFunction;
        use crate::provider::blocking::ProvidableFunctionKind;
        use crate::ptr::FactoryPtr;

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

        type IUserManagerFactory = dyn Fn(Vec<i128>) -> TransientPtr<dyn IUserManager>;

        let mut di_container = DIContainer::new();

        let factory_func: &dyn Fn(&DIContainer) -> Box<IUserManagerFactory> = &|_| {
            Box::new(move |users| {
                let user_manager: TransientPtr<dyn IUserManager> =
                    TransientPtr::new(UserManager::new(users));

                user_manager
            })
        };

        let mut mock_provider = MockIProvider::new();

        mock_provider.expect_provide().returning_st(|_, _| {
            Ok(Providable::Function(
                Rc::new(CastableFunction::new(factory_func)),
                ProvidableFunctionKind::UserCalled,
            ))
        });

        di_container
            .binding_storage
            .set::<IUserManagerFactory>(BindingOptions::new(), Box::new(mock_provider));

        di_container
            .get::<IUserManagerFactory>()
            .unwrap()
            .factory()
            .unwrap();
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory_named()
    {
        use std::rc::Rc;

        use crate::castable_function::CastableFunction;
        use crate::provider::blocking::ProvidableFunctionKind;

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

        type IUserManagerFactory = dyn Fn(Vec<i128>) -> TransientPtr<dyn IUserManager>;

        let mut di_container = DIContainer::new();

        let factory_func: &dyn Fn(&DIContainer) -> Box<IUserManagerFactory> = &|_| {
            Box::new(move |users| {
                let user_manager: TransientPtr<dyn IUserManager> =
                    TransientPtr::new(UserManager::new(users));

                user_manager
            })
        };

        let mut mock_provider = MockIProvider::new();

        mock_provider.expect_provide().returning_st(|_, _| {
            Ok(Providable::Function(
                Rc::new(CastableFunction::new(factory_func)),
                ProvidableFunctionKind::UserCalled,
            ))
        });

        di_container.binding_storage.set::<IUserManagerFactory>(
            BindingOptions::new().name("special"),
            Box::new(mock_provider),
        );

        di_container
            .get_named::<IUserManagerFactory>("special")
            .unwrap()
            .factory()
            .unwrap();
    }

    #[test]
    fn has_binding_works()
    {
        let mut di_container = DIContainer::new();

        // No binding is present yet
        assert!(!di_container.has_binding::<subjects::Ninja>(BindingOptions::new()));

        di_container.binding_storage.set::<subjects::Ninja>(
            BindingOptions::new(),
            Box::new(MockIProvider::new()),
        );

        assert!(di_container.has_binding::<subjects::Ninja>(BindingOptions::new()));
    }

    #[test]
    fn set_binding_works()
    {
        let mut di_container = DIContainer::new();

        di_container.set_binding::<subjects::Ninja>(
            BindingOptions::new(),
            Box::new(MockIProvider::new()),
        );

        assert!(di_container
            .binding_storage
            .has::<subjects::Ninja>(BindingOptions::new()));
    }

    #[test]
    fn remove_binding_works()
    {
        let mut di_container = DIContainer::new();

        di_container.binding_storage.set::<subjects::Ninja>(
            BindingOptions::new(),
            Box::new(MockIProvider::new()),
        );

        assert!(
            // Formatting is weird without this comment
            di_container
                .remove_binding::<subjects::Ninja>(BindingOptions::new())
                .is_some()
        );

        assert!(
            // Formatting is weird without this comment
            !di_container
                .binding_storage
                .has::<subjects::Ninja>(BindingOptions::new())
        );
    }
}
