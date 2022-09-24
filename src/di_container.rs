//! Dependency injection container.
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
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[cfg(feature = "factory")]
use crate::castable_factory::blocking::CastableFactory;
use crate::di_container_binding_map::DIContainerBindingMap;
use crate::errors::di_container::{
    BindingBuilderError,
    BindingScopeConfiguratorError,
    BindingWhenConfiguratorError,
    DIContainerError,
};
use crate::interfaces::injectable::Injectable;
use crate::libs::intertrait::cast::{CastBox, CastRc};
use crate::provider::blocking::{
    IProvider,
    Providable,
    SingletonProvider,
    TransientTypeProvider,
};
use crate::ptr::{SingletonPtr, SomePtr, TransientPtr};

/// When configurator for a binding for type 'Interface' inside a [`DIContainer`].
pub struct BindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: Rc<DIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> BindingWhenConfigurator<Interface>
where
    Interface: 'static + ?Sized,
{
    fn new(di_container: Rc<DIContainer>) -> Self
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
        &self,
        name: &'static str,
    ) -> Result<(), BindingWhenConfiguratorError>
    {
        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        let binding = bindings_mut.remove::<Interface>(None).map_or_else(
            || {
                Err(BindingWhenConfiguratorError::BindingNotFound(type_name::<
                    Interface,
                >(
                )))
            },
            Ok,
        )?;

        bindings_mut.set::<Interface>(Some(name), binding);

        Ok(())
    }
}

/// Scope configurator for a binding for type 'Interface' inside a [`DIContainer`].
pub struct BindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized,
    Implementation: Injectable,
{
    di_container: Rc<DIContainer>,
    interface_phantom: PhantomData<Interface>,
    implementation_phantom: PhantomData<Implementation>,
}

impl<Interface, Implementation> BindingScopeConfigurator<Interface, Implementation>
where
    Interface: 'static + ?Sized,
    Implementation: Injectable,
{
    fn new(di_container: Rc<DIContainer>) -> Self
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
    #[allow(clippy::must_use_candidate)]
    pub fn in_transient_scope(&self) -> BindingWhenConfigurator<Interface>
    {
        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        bindings_mut.set::<Interface>(
            None,
            Box::new(TransientTypeProvider::<Implementation>::new()),
        );

        BindingWhenConfigurator::new(self.di_container.clone())
    }

    /// Configures the binding to be in a singleton scope.
    ///
    /// # Errors
    /// Will return Err if resolving the implementation fails.
    pub fn in_singleton_scope(
        &self,
    ) -> Result<BindingWhenConfigurator<Interface>, BindingScopeConfiguratorError>
    {
        let singleton: SingletonPtr<Implementation> = SingletonPtr::from(
            Implementation::resolve(&self.di_container, Vec::new())
                .map_err(BindingScopeConfiguratorError::SingletonResolveFailed)?,
        );

        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        bindings_mut.set::<Interface>(None, Box::new(SingletonProvider::new(singleton)));

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
    }
}

/// Binding builder for type `Interface` inside a [`DIContainer`].
pub struct BindingBuilder<Interface>
where
    Interface: 'static + ?Sized,
{
    di_container: Rc<DIContainer>,
    interface_phantom: PhantomData<Interface>,
}

impl<Interface> BindingBuilder<Interface>
where
    Interface: 'static + ?Sized,
{
    fn new(di_container: Rc<DIContainer>) -> Self
    {
        Self {
            di_container,
            interface_phantom: PhantomData,
        }
    }

    /// Creates a binding of type `Interface` to type `Implementation` inside of the
    /// associated [`DIContainer`].
    ///
    /// The scope of the binding is transient. But that can be changed by using the
    /// returned [`BindingScopeConfigurator`]
    ///
    /// # Errors
    /// Will return Err if the associated [`DIContainer`] already have a binding for
    /// the interface.
    pub fn to<Implementation>(
        &self,
    ) -> Result<BindingScopeConfigurator<Interface, Implementation>, BindingBuilderError>
    where
        Implementation: Injectable,
    {
        {
            let bindings = self.di_container.bindings.borrow();

            if bindings.has::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let binding_scope_configurator =
            BindingScopeConfigurator::new(self.di_container.clone());

        binding_scope_configurator.in_transient_scope();

        Ok(binding_scope_configurator)
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
        &self,
        factory_func: &'static dyn Fn<
            (std::rc::Rc<DIContainer>,),
            Output = Box<dyn Fn<Args, Output = crate::ptr::TransientPtr<Return>>>,
        >,
    ) -> Result<BindingWhenConfigurator<Interface>, BindingBuilderError>
    where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: Fn<Args, Output = crate::ptr::TransientPtr<Return>>,
    {
        {
            let bindings = self.di_container.bindings.borrow();

            if bindings.has::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        let factory_impl = CastableFactory::new(factory_func);

        bindings_mut.set::<Interface>(
            None,
            Box::new(crate::provider::blocking::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
                false,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
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
    pub fn to_default_factory<Return, FactoryFunc>(
        &self,
        factory_func: &'static FactoryFunc,
    ) -> Result<BindingWhenConfigurator<Interface>, BindingBuilderError>
    where
        Return: 'static + ?Sized,
        FactoryFunc: Fn<
            (Rc<DIContainer>,),
            Output = crate::ptr::TransientPtr<
                dyn Fn<(), Output = crate::ptr::TransientPtr<Return>>,
            >,
        >,
    {
        {
            let bindings = self.di_container.bindings.borrow();

            if bindings.has::<Interface>(None) {
                return Err(BindingBuilderError::BindingAlreadyExists(type_name::<
                    Interface,
                >(
                )));
            }
        }

        let mut bindings_mut = self.di_container.bindings.borrow_mut();

        let factory_impl = CastableFactory::new(factory_func);

        bindings_mut.set::<Interface>(
            None,
            Box::new(crate::provider::blocking::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
                true,
            )),
        );

        Ok(BindingWhenConfigurator::new(self.di_container.clone()))
    }
}

/// Dependency injection container.
pub struct DIContainer
{
    bindings: RefCell<DIContainerBindingMap<dyn IProvider>>,
}

impl DIContainer
{
    /// Returns a new `DIContainer`.
    #[must_use]
    pub fn new() -> Rc<Self>
    {
        Rc::new(Self {
            bindings: RefCell::new(DIContainerBindingMap::new()),
        })
    }

    /// Returns a new [`BindingBuilder`] for the given interface.
    #[must_use]
    pub fn bind<Interface>(self: &mut Rc<Self>) -> BindingBuilder<Interface>
    where
        Interface: 'static + ?Sized,
    {
        BindingBuilder::<Interface>::new(self.clone())
    }

    /// Returns the type bound with `Interface`.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` exists
    /// - Resolving the binding for fails
    /// - Casting the binding for fails
    pub fn get<Interface>(self: &Rc<Self>) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(Vec::new(), None)
    }

    /// Returns the type bound with `Interface` and the specified name.
    ///
    /// # Errors
    /// Will return `Err` if:
    /// - No binding for `Interface` with name `name` exists
    /// - Resolving the binding fails
    /// - Casting the binding for fails
    pub fn get_named<Interface>(
        self: &Rc<Self>,
        name: &'static str,
    ) -> Result<SomePtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.get_bound::<Interface>(Vec::new(), Some(name))
    }

    #[doc(hidden)]
    pub fn get_bound<Interface>(
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

    fn handle_binding_providable<Interface>(
        self: &Rc<Self>,
        binding_providable: Providable,
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
    ) -> Result<Providable, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        self.bindings
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

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use mockall::mock;

    use super::*;
    use crate::errors::injectable::InjectableError;
    use crate::provider::blocking::IProvider;
    use crate::ptr::TransientPtr;

    mod subjects
    {
        //! Test subjects.

        use std::fmt::Debug;
        use std::rc::Rc;

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
                _di_container: &Rc<DIContainer>,
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
                _di_container: &Rc<DIContainer>,
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
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_transient() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_transient_scope();

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_transient_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_transient_scope()
            .when_named("regular")?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_singleton() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_singleton_scope()?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_bind_to_singleton_when_named() -> Result<(), Box<dyn Error>>
    {
        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<dyn subjects::IUserManager>()
            .to::<subjects::UserManager>()?
            .in_singleton_scope()?
            .when_named("cool")?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory() -> Result<(), Box<dyn Error>>
    {
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(), dyn subjects::IUserManager>;

        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(move || {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

                    user_manager
                })
            })?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory_when_named() -> Result<(), Box<dyn Error>>
    {
        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(), dyn subjects::IUserManager>;

        let mut di_container = DIContainer::new();

        assert_eq!(di_container.bindings.borrow().count(), 0);

        di_container
            .bind::<IUserManagerFactory>()
            .to_factory(&|_| {
                Box::new(move || {
                    let user_manager: TransientPtr<dyn subjects::IUserManager> =
                        TransientPtr::new(subjects::UserManager::new());

                    user_manager
                })
            })?
            .when_named("awesome")?;

        assert_eq!(di_container.bindings.borrow().count(), 1);

        Ok(())
    }

    #[test]
    fn can_get() -> Result<(), Box<dyn Error>>
    {
        mock! {
            Provider {}

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
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
            .bindings
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

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
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
            .bindings
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

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
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
            .bindings
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

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
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
            .bindings
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

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
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
            .bindings
            .borrow_mut()
            .set::<IUserManagerFactory>(None, Box::new(mock_provider));

        di_container.get::<IUserManagerFactory>()?.factory()?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory_named() -> Result<(), Box<dyn Error>>
    {
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

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &Rc<DIContainer>,
                    dependency_history: Vec<&'static str>,
                ) -> Result<Providable, InjectableError>;
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
            .bindings
            .borrow_mut()
            .set::<IUserManagerFactory>(Some("special"), Box::new(mock_provider));

        di_container
            .get_named::<IUserManagerFactory>("special")?
            .factory()?;

        Ok(())
    }
}
