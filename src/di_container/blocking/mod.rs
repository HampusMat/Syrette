//! Blocking dependency injection container.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use std::error::Error;
//!
//! use syrette::di_container::blocking::IDIContainer;
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
use std::cell::RefCell;
use std::rc::Rc;

use crate::di_container::binding_storage::DIContainerBindingStorage;
use crate::di_container::blocking::binding::builder::BindingBuilder;
use crate::errors::di_container::DIContainerError;
use crate::libs::intertrait::cast::{CastBox, CastRc};
use crate::provider::blocking::{IProvider, Providable};
use crate::ptr::SomePtr;

pub mod binding;
pub mod prelude;

/// Blocking dependency injection container interface.
pub trait IDIContainer: Sized + 'static + details::DIContainerInternals
{
    /// Returns a new [`BindingBuilder`] for the given interface.
    fn bind<Interface>(self: &mut Rc<Self>) -> BindingBuilder<Interface, Self>
    where
        Interface: 'static + ?Sized;

    /// Returns the type bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    fn get<Interface>(self: &Rc<Self>) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized;

    /// Returns the type bound with `Interface` and the specified name.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` with name `name` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    fn get_named<Interface>(
        self: &Rc<Self>,
        name: &'static str,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized;

    #[doc(hidden)]
    fn get_bound<Interface>(
        self: &Rc<Self>,
        dependency_history: Vec<&'static str>,
        name: Option<&'static str>,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized;
}

/// Blocking dependency injection container.
pub struct DIContainer
{
    binding_storage: RefCell<DIContainerBindingStorage<dyn IProvider<Self>>>,
}

impl DIContainer
{
    /// Returns a new `DIContainer`.
    #[must_use]
    pub fn new() -> Rc<Self>
    {
        Rc::new(Self {
            binding_storage: RefCell::new(DIContainerBindingStorage::new()),
        })
    }
}

impl IDIContainer for DIContainer
{
    #[must_use]
    fn bind<Interface>(self: &mut Rc<Self>) -> BindingBuilder<Interface, Self>
    where
        Interface: 'static + ?Sized,
    {
        BindingBuilder::<Interface, Self>::new(self.clone())
    }

    fn get<Interface>(self: &Rc<Self>) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(Vec::new(), None)
    }

    fn get_named<Interface>(
        self: &Rc<Self>,
        name: &'static str,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(Vec::new(), Some(name))
    }

    #[doc(hidden)]
    fn get_bound<Interface>(
        self: &Rc<Self>,
        dependency_history: Vec<&'static str>,
        name: Option<&'static str>,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let binding_providable =
            self.get_binding_providable::<Interface>(name, dependency_history)?;

        self.handle_binding_providable(binding_providable)
    }
}

impl details::DIContainerInternals for DIContainer
{
    fn has_binding<Interface>(self: &Rc<Self>, name: Option<&'static str>) -> bool
    where
        Interface: ?Sized + 'static,
    {
        self.binding_storage.borrow().has::<Interface>(name)
    }

    fn set_binding<Interface>(
        self: &Rc<Self>,
        name: Option<&'static str>,
        provider: Box<dyn IProvider<Self>>,
    ) where
        Interface: 'static + ?Sized,
    {
        self.binding_storage
            .borrow_mut()
            .set::<Interface>(name, provider);
    }

    fn remove_binding<Interface>(
        self: &Rc<Self>,
        name: Option<&'static str>,
    ) -> Option<Box<dyn IProvider<Self>>>
    where
        Interface: 'static + ?Sized,
    {
        self.binding_storage.borrow_mut().remove::<Interface>(name)
    }
}

impl DIContainer
{
    fn handle_binding_providable<Interface>(
        self: &Rc<Self>,
        binding_providable: Providable<Self>,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
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
            Providable::Factory(factory_binding) => {
                use crate::interfaces::factory::IFactory;

                let factory = factory_binding
                    .cast::<dyn IFactory<(Rc<DIContainer>,), Interface>>()
                    .map_err(|_| DIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "factory",
                    })?;

                Ok(SomePtr::Factory(factory(self.clone()).into()))
            }
            #[cfg(feature = "factory")]
            Providable::DefaultFactory(factory_binding) => {
                use crate::interfaces::factory::IFactory;
                use crate::ptr::TransientPtr;

                let default_factory = factory_binding
                    .cast::<dyn IFactory<
                        (Rc<DIContainer>,),
                        dyn Fn<(), Output = TransientPtr<Interface>>,
                    >>()
                    .map_err(|_| DIContainerError::CastFailed {
                        interface: type_name::<Interface>(),
                        binding_kind: "default factory",
                    })?;

                Ok(SomePtr::Transient(default_factory(self.clone())()))
            }
        }
    }

    fn get_binding_providable<Interface>(
        self: &Rc<Self>,
        name: Option<&'static str>,
        dependency_history: Vec<&'static str>,
    ) -> Result<Providable<Self>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.binding_storage
            .borrow()
            .get::<Interface>(name)
            .map_or_else(
                || {
                    Err(DIContainerError::BindingNotFound {
                        interface: type_name::<Interface>(),
                        name,
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

pub(crate) mod details
{
    use std::rc::Rc;

    use crate::provider::blocking::IProvider;

    pub trait DIContainerInternals
    {
        fn has_binding<Interface>(self: &Rc<Self>, name: Option<&'static str>) -> bool
        where
            Interface: ?Sized + 'static;

        fn set_binding<Interface>(
            self: &Rc<Self>,
            name: Option<&'static str>,
            provider: Box<dyn IProvider<Self>>,
        ) where
            Interface: 'static + ?Sized;

        fn remove_binding<Interface>(
            self: &Rc<Self>,
            name: Option<&'static str>,
        ) -> Option<Box<dyn IProvider<Self>>>
        where
            Interface: 'static + ?Sized;
    }
}

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use mockall::mock;

    use super::*;
    use crate::errors::injectable::InjectableError;
    use crate::provider::blocking::IProvider;
    use crate::ptr::{SingletonPtr, TransientPtr};
    use crate::test_utils::subjects;

    #[test]
    fn can_get() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            impl IProvider<DIContainer> for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable<DIContainer>, InjectableError>;
            }
        }

        let di_container = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(Providable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .binding_storage
            .borrow_mut()
            .set::<dyn subjects::IUserManager>(None, Box::new(mock_provider));

        di_container
            .get::<dyn subjects::IUserManager>()?
            .transient()?;

        Ok(())
    }

    #[test]
    fn can_get_named() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            impl IProvider<DIContainer> for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable<DIContainer>, InjectableError>;
            }
        }

        let di_container = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(Providable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .binding_storage
            .borrow_mut()
            .set::<dyn subjects::IUserManager>(Some("special"), Box::new(mock_provider));

        di_container
            .get_named::<dyn subjects::IUserManager>("special")?
            .transient()?;

        Ok(())
    }

    #[test]
    fn can_get_singleton() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            impl IProvider<DIContainer> for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable<DIContainer>, InjectableError>;
            }
        }

        let di_container = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = SingletonPtr::new(subjects::Number::new());

        SingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(Providable::Singleton(singleton.clone())));

        di_container
            .binding_storage
            .borrow_mut()
            .set::<dyn subjects::INumber>(None, Box::new(mock_provider));

        let first_number_rc = di_container.get::<dyn subjects::INumber>()?.singleton()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc =
            di_container.get::<dyn subjects::INumber>()?.singleton()?;

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());

        Ok(())
    }

    #[test]
    fn can_get_singleton_named() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            impl IProvider<DIContainer> for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable<DIContainer>, InjectableError>;
            }
        }

        let di_container = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = SingletonPtr::new(subjects::Number::new());

        SingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(Providable::Singleton(singleton.clone())));

        di_container
            .binding_storage
            .borrow_mut()
            .set::<dyn subjects::INumber>(Some("cool"), Box::new(mock_provider));

        let first_number_rc = di_container
            .get_named::<dyn subjects::INumber>("cool")?
            .singleton()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container
            .get_named::<dyn subjects::INumber>("cool")?
            .singleton()?;

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory() -> Result<(), Box<dyn Error>>
    {
        use crate::castable_factory::blocking::CastableFactory;
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

        type FactoryFunc = dyn Fn<
            (std::rc::Rc<DIContainer>,),
            Output = Box<
                dyn Fn<(Vec<i128>,), Output = crate::ptr::TransientPtr<dyn IUserManager>>,
            >,
        >;

        use crate as syrette;

        #[crate::factory]
        type IUserManagerFactory = dyn Fn(Vec<i128>) -> dyn IUserManager;

        mock! {
            Provider {}

            impl IProvider<DIContainer> for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable<DIContainer>, InjectableError>;
            }
        }

        let di_container = DIContainer::new();

        let factory_func: &'static FactoryFunc = &|_: Rc<DIContainer>| {
            Box::new(move |users| {
                let user_manager: TransientPtr<dyn IUserManager> =
                    TransientPtr::new(UserManager::new(users));

                user_manager
            })
        };

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning_st(|_, _| {
            Ok(Providable::Factory(FactoryPtr::new(CastableFactory::new(
                factory_func,
            ))))
        });

        di_container
            .binding_storage
            .borrow_mut()
            .set::<IUserManagerFactory>(None, Box::new(mock_provider));

        di_container.get::<IUserManagerFactory>()?.factory()?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory_named() -> Result<(), Box<dyn Error>>
    {
        use crate::castable_factory::blocking::CastableFactory;
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

        type FactoryFunc = dyn Fn<
            (std::rc::Rc<DIContainer>,),
            Output = Box<
                dyn Fn<(Vec<i128>,), Output = crate::ptr::TransientPtr<dyn IUserManager>>,
            >,
        >;

        use crate as syrette;

        #[crate::factory]
        type IUserManagerFactory = dyn Fn(Vec<i128>) -> dyn IUserManager;

        mock! {
            Provider {}

            impl IProvider<DIContainer> for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable<DIContainer>, InjectableError>;
            }
        }

        let di_container = DIContainer::new();

        let factory_func: &'static FactoryFunc = &|_: Rc<DIContainer>| {
            Box::new(move |users| {
                let user_manager: TransientPtr<dyn IUserManager> =
                    TransientPtr::new(UserManager::new(users));

                user_manager
            })
        };

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning_st(|_, _| {
            Ok(Providable::Factory(FactoryPtr::new(CastableFactory::new(
                factory_func,
            ))))
        });

        di_container
            .binding_storage
            .borrow_mut()
            .set::<IUserManagerFactory>(Some("special"), Box::new(mock_provider));

        di_container
            .get_named::<IUserManagerFactory>("special")?
            .factory()?;

        Ok(())
    }
}
