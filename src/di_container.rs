//! Dependency injection container and other related utilities.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//!
//! use syrette::{DIContainer, injectable};
//! use syrette::errors::di_container::DIContainerError;
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
//! fn main() -> Result<(), String>
//! {
//!     let mut di_container = DIContainer::new();
//!
//!     di_container.bind::<dyn IDatabaseService>().to::<DatabaseService>().map_err(|err| {
//!         err.to_string()
//!     })?;
//!
//!     let database_service = di_container.get::<dyn IDatabaseService>().map_err(|err| {
//!         err.to_string()
//!     })?;
//!
//!     Ok(())
//! }
//! ```
use std::any::type_name;
use std::marker::PhantomData;

#[cfg(feature = "factory")]
use crate::castable_factory::CastableFactory;
use crate::di_container_binding_map::DIContainerBindingMap;
use crate::errors::di_container::{BindingBuilderError, DIContainerError};
use crate::interfaces::injectable::Injectable;
use crate::libs::intertrait::cast::{CastBox, CastRc};
use crate::provider::{Providable, SingletonProvider, TransientTypeProvider};
use crate::ptr::{SingletonPtr, TransientPtr};

/// Binding builder for type `Interface` inside a [`DIContainer`].
pub struct BindingBuilder<'di_container_lt, Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: &'di_container_lt mut DIContainer,
    interface_phantom: PhantomData<Interface>,
}

impl<'di_container_lt, Interface> BindingBuilder<'di_container_lt, Interface>
where
    Interface: 'static + ?Sized,
{
    fn new(di_container: &'di_container_lt mut DIContainer) -> Self
    {
        Self {
            di_container,
            interface_phantom: PhantomData,
        }
    }

    /// Creates a binding of type `Interface` to type `Implementation` inside of the
    /// associated [`DIContainer`].
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    pub fn to<Implementation>(&mut self) -> Result<(), BindingBuilderError>
    where
        Implementation: Injectable,
    {
        self.di_container
            .bindings
            .set::<Interface>(Box::new(TransientTypeProvider::<Implementation>::new()))
            .ok_or_else(|| {
                BindingBuilderError::BindingAlreadyExists(type_name::<Interface>())
            })?;

        Ok(())
    }

    /// Creates a binding of type `Interface` to a new singleton of type `Implementation`
    /// inside of the associated [`DIContainer`].
    ///
    /// # Errors
    /// Will return Err if creating the singleton fails or if the
    /// associated [`DIContainer`] already have a binding for the interface.
    pub fn to_singleton<Implementation>(&mut self) -> Result<(), BindingBuilderError>
    where
        Implementation: Injectable,
    {
        let singleton: SingletonPtr<Implementation> = SingletonPtr::from(
            Implementation::resolve(self.di_container, Vec::new())
                .map_err(BindingBuilderError::SingletonResolveFailed)?,
        );

        self.di_container
            .bindings
            .set::<Interface>(Box::new(SingletonProvider::new(singleton)))
            .ok_or_else(|| {
                BindingBuilderError::BindingAlreadyExists(type_name::<Interface>())
            })?;

        Ok(())
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`DIContainer`].
    ///
    /// *This function is only available if Syrette is built with the "factory" feature.*
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    #[cfg(feature = "factory")]
    pub fn to_factory<Args, Return>(
        &mut self,
        factory_func: &'static dyn Fn<Args, Output = TransientPtr<Return>>,
    ) -> Result<(), BindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: crate::interfaces::factory::IFactory<Args, Return>,
    {
        let factory_impl = CastableFactory::new(factory_func);

        self.di_container
            .bindings
            .set::<Interface>(Box::new(crate::provider::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
            )))
            .ok_or_else(|| {
                BindingBuilderError::BindingAlreadyExists(type_name::<Interface>())
            })?;

        Ok(())
    }

    /// Creates a binding of type `Interface` to a factory that takes no arguments
    /// inside of the associated [`DIContainer`].
    ///
    /// *This function is only available if Syrette is built with the "factory" feature.*
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    #[cfg(feature = "factory")]
    pub fn to_default_factory<Return>(
        &mut self,
        factory_func: &'static dyn Fn<(), Output = TransientPtr<Return>>,
    ) -> Result<(), BindingBuilderError>
    where
        Return: 'static + ?Sized,
    {
        let factory_impl = CastableFactory::new(factory_func);

        self.di_container
            .bindings
            .set::<Interface>(Box::new(crate::provider::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
            )))
            .ok_or_else(|| {
                BindingBuilderError::BindingAlreadyExists(type_name::<Interface>())
            })?;

        Ok(())
    }
}

/// Dependency injection container.
pub struct DIContainer
{
    bindings: DIContainerBindingMap,
}

impl DIContainer
{
    /// Returns a new `DIContainer`.
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            bindings: DIContainerBindingMap::new(),
        }
    }

    /// Returns a new [`BindingBuilder`] for the given interface.
    pub fn bind<Interface>(&mut self) -> BindingBuilder<Interface>
    where
        Interface: 'static + ?Sized,
    {
        BindingBuilder::<Interface>::new(self)
    }

    /// Returns a new instance of the type bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    /// - The binding for `Interface` is not transient
    pub fn get<Interface>(&self) -> Result<TransientPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_with_history::<Interface>(Vec::new())
    }

    /// Returns the singleton instance bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    /// - The binding for `Interface` is not a singleton
    pub fn get_singleton<Interface>(
        &self,
    ) -> Result<SingletonPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_singleton_with_history(Vec::new())
    }

    /// Returns the factory bound with factory type `Interface`.
    ///
    /// *This function is only available if Syrette is built with the "factory" feature.*
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for `Interface` fails
    /// - Casting the binding for `Interface` fails
    /// - The binding for `Interface` is not a factory
    #[cfg(feature = "factory")]
    pub fn get_factory<Interface>(
        &self,
    ) -> Result<crate::ptr::FactoryPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let binding_providable = self.get_binding_providable::<Interface>(Vec::new())?;

        if let Providable::Factory(binding_factory) = binding_providable {
            return binding_factory
                .cast::<Interface>()
                .map_err(|_| DIContainerError::CastFailed(type_name::<Interface>()));
        }

        Err(DIContainerError::WrongBindingType {
            interface: type_name::<Interface>(),
            expected: "factory",
            found: binding_providable.to_string().to_lowercase(),
        })
    }

    #[doc(hidden)]
    pub fn get_with_history<Interface>(
        &self,
        dependency_history: Vec<&'static str>,
    ) -> Result<TransientPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let binding_providable =
            self.get_binding_providable::<Interface>(dependency_history)?;

        if let Providable::Transient(binding_transient) = binding_providable {
            return binding_transient
                .cast::<Interface>()
                .map_err(|_| DIContainerError::CastFailed(type_name::<Interface>()));
        }

        #[cfg(feature = "factory")]
        if let Providable::Factory(binding_factory) = binding_providable {
            use crate::interfaces::factory::IFactory;

            let factory = binding_factory
                .cast::<dyn IFactory<(), Interface>>()
                .map_err(|_| DIContainerError::CastFailed(type_name::<Interface>()))?;

            return Ok(factory());
        }

        Err(DIContainerError::WrongBindingType {
            interface: type_name::<Interface>(),
            expected: "transient",
            found: binding_providable.to_string().to_lowercase(),
        })
    }

    #[doc(hidden)]
    pub fn get_singleton_with_history<Interface>(
        &self,
        dependency_history: Vec<&'static str>,
    ) -> Result<SingletonPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let binding_providable =
            self.get_binding_providable::<Interface>(dependency_history)?;

        if let Providable::Singleton(binding_singleton) = binding_providable {
            return binding_singleton
                .cast::<Interface>()
                .map_err(|_| DIContainerError::CastFailed(type_name::<Interface>()));
        }

        Err(DIContainerError::WrongBindingType {
            interface: type_name::<Interface>(),
            expected: "singleton",
            found: binding_providable.to_string().to_lowercase(),
        })
    }

    fn get_binding_providable<Interface>(
        &self,
        dependency_history: Vec<&'static str>,
    ) -> Result<Providable, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.bindings
            .get::<Interface>()?
            .provide(self, dependency_history)
            .map_err(|err| DIContainerError::BindingResolveFailed {
                reason: err,
                interface: type_name::<Interface>(),
            })
    }
}

impl Default for DIContainer
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use mockall::mock;

    use super::*;
    use crate::errors::injectable::InjectableError;
    use crate::provider::IProvider;
    use crate::ptr::TransientPtr;

    mod subjects
    {
        //! Test subjects.

        use std::fmt::Debug;

        use syrette_macros::declare_interface;

        use super::DIContainer;
        use crate::interfaces::injectable::Injectable;
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

        impl Injectable for UserManager
        {
            fn resolve(
                _di_container: &DIContainer,
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

        declare_interface!(Number -> INumber);

        impl Injectable for Number
        {
            fn resolve(
                _di_container: &DIContainer,
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
    fn can_bind_to() -> Result<(), BindingBuilderError>
    {
        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_singleton() -> Result<(), BindingBuilderError>
    {
        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to_singleton::<subjects::UserManager>()?;

        assert_eq!(di_container.bindings.count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory() -> Result<(), BindingBuilderError>
    {
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(), dyn subjects::IUserManager>;

        let mut di_container: DIContainer = DIContainer::new();

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
    fn can_get() -> Result<(), DIContainerError>
    {
        mock! {
            Provider {}

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &DIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(Providable::Transient(TransientPtr::new(
                subjects::UserManager::new(),
            )))
        });

        di_container
            .bindings
            .set::<dyn subjects::IUserManager>(Box::new(mock_provider));

        di_container.get::<dyn subjects::IUserManager>()?;

        Ok(())
    }

    #[test]
    fn can_get_singleton() -> Result<(), DIContainerError>
    {
        mock! {
            Provider {}

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &DIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = SingletonPtr::new(subjects::Number::new());

        SingletonPtr::get_mut(&mut singleton).unwrap().num = 2820;

        mock_provider
            .expect_provide()
            .returning_st(move |_, _| Ok(Providable::Singleton(singleton.clone())));

        di_container
            .bindings
            .set::<dyn subjects::INumber>(Box::new(mock_provider));

        let first_number_rc = di_container.get_singleton::<dyn subjects::INumber>()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container.get_singleton::<dyn subjects::INumber>()?;

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory() -> Result<(), DIContainerError>
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

        #[crate::factory]
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(Vec<i128>,), dyn IUserManager>;

        mock! {
            Provider {}

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &DIContainer,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_, _| {
            Ok(Providable::Factory(crate::ptr::FactoryPtr::new(
                CastableFactory::new(&|users| {
                    let user_manager: TransientPtr<dyn IUserManager> =
                        TransientPtr::new(UserManager::new(users));

                    user_manager
                }),
            )))
        });

        di_container
            .bindings
            .set::<IUserManagerFactory>(Box::new(mock_provider));

        di_container.get_factory::<IUserManagerFactory>()?;

        Ok(())
    }
}
