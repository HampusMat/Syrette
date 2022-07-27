use std::any::{type_name, TypeId};
use std::marker::PhantomData;
use std::rc::Rc;

use ahash::AHashMap;
use error_stack::{Report, ResultExt};

#[cfg(feature = "factory")]
use crate::castable_factory::CastableFactory;
use crate::errors::di_container::{BindingBuilderError, DIContainerError};
use crate::interfaces::injectable::Injectable;
use crate::libs::intertrait::cast::{CastBox, CastRc};
use crate::provider::{IProvider, Providable, SingletonProvider, TransientTypeProvider};
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
    pub fn to<Implementation>(&mut self)
    where
        Implementation: Injectable,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self.di_container.bindings.insert(
            interface_typeid,
            Rc::new(TransientTypeProvider::<Implementation>::new()),
        );
    }

    /// Creates a binding of type `Interface` to a new singleton of type `Implementation`
    /// inside of the associated [`DIContainer`].
    ///
    /// # Errors
    /// Will return Err if creating the singleton fails.
    pub fn to_singleton<Implementation>(
        &mut self,
    ) -> error_stack::Result<(), BindingBuilderError>
    where
        Implementation: Injectable,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let singleton: SingletonPtr<Implementation> = SingletonPtr::from(
            Implementation::resolve(self.di_container)
                .change_context(BindingBuilderError)?,
        );

        self.di_container
            .bindings
            .insert(interface_typeid, Rc::new(SingletonProvider::new(singleton)));

        Ok(())
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`DIContainer`].
    #[cfg(feature = "factory")]
    pub fn to_factory<Args, Return>(
        &mut self,
        factory_func: &'static dyn Fn<Args, Output = TransientPtr<Return>>,
    ) where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: crate::interfaces::factory::IFactory<Args, Return>,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let factory_impl = CastableFactory::new(factory_func);

        self.di_container.bindings.insert(
            interface_typeid,
            Rc::new(crate::provider::FactoryProvider::new(
                crate::ptr::FactoryPtr::new(factory_impl),
            )),
        );
    }
}

/// Dependency injection container.
///
/// # Examples
/// ```
/// use std::collections::HashMap;
///
/// use syrette::{DIContainer, injectable};
/// use syrette::errors::di_container::DIContainerError;
///
/// trait IDatabaseService
/// {
///     fn get_all_records(&self, table_name: String) -> HashMap<String, String>;
/// }
///
/// struct DatabaseService {}
///
/// #[injectable(IDatabaseService)]
/// impl DatabaseService
/// {
///     fn new() -> Self
///     {
///         Self {}
///     }
/// }
///
/// impl IDatabaseService for DatabaseService
/// {
///     fn get_all_records(&self, table_name: String) -> HashMap<String, String>
///     {
///         // Do stuff here
///         HashMap::<String, String>::new()
///     }
/// }
///
/// fn main() -> error_stack::Result<(), DIContainerError>
/// {
///     let mut di_container = DIContainer::new();
///
///     di_container.bind::<dyn IDatabaseService>().to::<DatabaseService>();
///
///     let database_service = di_container.get::<dyn IDatabaseService>()?;
///
///     Ok(())
/// }
/// ```
pub struct DIContainer
{
    bindings: AHashMap<TypeId, Rc<dyn IProvider>>,
}

impl DIContainer
{
    /// Returns a new `DIContainer`.
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            bindings: AHashMap::new(),
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
    /// - The binding for `Interface` is not injectable
    pub fn get<Interface>(
        &self,
    ) -> error_stack::Result<TransientPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let interface_name = type_name::<Interface>();

        let binding = self.bindings.get(&interface_typeid).ok_or_else(|| {
            Report::new(DIContainerError)
                .attach_printable(format!("No binding exists for {}", interface_name))
        })?;

        let binding_providable = binding
            .provide(self)
            .change_context(DIContainerError)
            .attach_printable(format!(
            "Failed to resolve binding for interface {}",
            interface_name
        ))?;

        match binding_providable {
            Providable::Transient(binding_injectable) => {
                let interface_result = binding_injectable.cast::<Interface>();

                match interface_result {
                    Ok(interface) => Ok(interface),
                    Err(_) => Err(Report::new(DIContainerError).attach_printable(
                        format!("Unable to cast binding for {}", interface_name),
                    )),
                }
            }
            _ => Err(Report::new(DIContainerError).attach_printable(format!(
                "Binding for {} is not injectable",
                interface_name
            ))),
        }
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
    ) -> error_stack::Result<SingletonPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let interface_name = type_name::<Interface>();

        let binding = self.bindings.get(&interface_typeid).ok_or_else(|| {
            Report::new(DIContainerError)
                .attach_printable(format!("No binding exists for {}", interface_name))
        })?;

        let binding_providable = binding
            .provide(self)
            .change_context(DIContainerError)
            .attach_printable(format!(
            "Failed to resolve binding for interface {}",
            interface_name
        ))?;

        match binding_providable {
            Providable::Singleton(binding_singleton) => {
                let interface_result = binding_singleton.cast::<Interface>();

                match interface_result {
                    Ok(interface) => Ok(interface),
                    Err(_) => Err(Report::new(DIContainerError).attach_printable(
                        format!("Unable to cast binding for {}", interface_name),
                    )),
                }
            }
            _ => Err(Report::new(DIContainerError).attach_printable(format!(
                "Binding for {} is not a singleton",
                interface_name
            ))),
        }
    }

    /// Returns the factory bound with factory type `Interface`.
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
    ) -> error_stack::Result<crate::ptr::FactoryPtr<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let interface_name = type_name::<Interface>();

        let binding = self.bindings.get(&interface_typeid).ok_or_else(|| {
            Report::new(DIContainerError)
                .attach_printable(format!("No binding exists for {}", interface_name))
        })?;

        let binding_providable = binding
            .provide(self)
            .change_context(DIContainerError)
            .attach_printable(format!(
            "Failed to resolve binding for interface {}",
            interface_name
        ))?;

        match binding_providable {
            Providable::Factory(binding_factory) => {
                let factory_result = binding_factory.cast::<Interface>();

                match factory_result {
                    Ok(factory) => Ok(factory),
                    Err(_) => Err(Report::new(DIContainerError).attach_printable(
                        format!("Unable to cast binding for {}", interface_name),
                    )),
                }
            }
            _ => Err(Report::new(DIContainerError).attach_printable(format!(
                "Binding for {} is not a factory",
                interface_name
            ))),
        }
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
    use std::fmt::Debug;

    use mockall::mock;

    use super::*;
    use crate::errors::injectable::ResolveError;
    use crate::ptr::TransientPtr;

    #[test]
    fn can_bind_to()
    {
        trait IUserManager
        {
            fn add_user(&self, user_id: i128);

            fn remove_user(&self, user_id: i128);
        }

        struct UserManager {}

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

        impl Injectable for UserManager
        {
            fn resolve(
                _di_container: &DIContainer,
            ) -> error_stack::Result<
                TransientPtr<Self>,
                crate::errors::injectable::ResolveError,
            >
            where
                Self: Sized,
            {
                Ok(TransientPtr::new(Self {}))
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.len(), 0);

        di_container.bind::<dyn IUserManager>().to::<UserManager>();

        assert_eq!(di_container.bindings.len(), 1);
    }

    #[test]
    fn can_bind_to_singleton() -> error_stack::Result<(), BindingBuilderError>
    {
        trait IUserManager
        {
            fn add_user(&self, user_id: i128);

            fn remove_user(&self, user_id: i128);
        }

        struct UserManager {}

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

        impl Injectable for UserManager
        {
            fn resolve(
                _di_container: &DIContainer,
            ) -> error_stack::Result<
                TransientPtr<Self>,
                crate::errors::injectable::ResolveError,
            >
            where
                Self: Sized,
            {
                Ok(TransientPtr::new(Self {}))
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.len(), 0);

        di_container
            .bind::<dyn IUserManager>()
            .to_singleton::<UserManager>()?;

        assert_eq!(di_container.bindings.len(), 1);

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_bind_to_factory()
    {
        trait IUserManager
        {
            fn add_user(&self, user_id: i128);

            fn remove_user(&self, user_id: i128);
        }

        struct UserManager {}

        impl UserManager
        {
            fn new() -> Self
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

        type IUserManagerFactory =
            dyn crate::interfaces::factory::IFactory<(), dyn IUserManager>;

        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.len(), 0);

        di_container.bind::<IUserManagerFactory>().to_factory(&|| {
            let user_manager: TransientPtr<dyn IUserManager> =
                TransientPtr::new(UserManager::new());

            user_manager
        });

        assert_eq!(di_container.bindings.len(), 1);
    }

    #[test]
    fn can_get() -> error_stack::Result<(), DIContainerError>
    {
        trait IUserManager
        {
            fn add_user(&self, user_id: i128);

            fn remove_user(&self, user_id: i128);
        }

        struct UserManager {}

        use crate as syrette;
        use crate::injectable;

        #[injectable(IUserManager)]
        impl UserManager
        {
            fn new() -> Self
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

        mock! {
            Provider {}

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &DIContainer,
                ) -> error_stack::Result<Providable, ResolveError>;
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_| {
            Ok(Providable::Transient(TransientPtr::new(UserManager::new())))
        });

        di_container
            .bindings
            .insert(TypeId::of::<dyn IUserManager>(), Rc::new(mock_provider));

        di_container.get::<dyn IUserManager>()?;

        Ok(())
    }

    #[test]
    fn can_get_singleton() -> error_stack::Result<(), DIContainerError>
    {
        trait INumber
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

        struct Number
        {
            num: i32,
        }

        use crate as syrette;
        use crate::injectable;

        #[injectable(INumber)]
        impl Number
        {
            fn new() -> Self
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

        mock! {
            Provider {}

            impl IProvider for Provider
            {
                fn provide(
                    &self,
                    di_container: &DIContainer,
                ) -> error_stack::Result<Providable, ResolveError>;
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        let mut singleton = SingletonPtr::new(Number::new());

        SingletonPtr::get_mut(&mut singleton).unwrap().set(2820);

        mock_provider
            .expect_provide()
            .returning_st(move |_| Ok(Providable::Singleton(singleton.clone())));

        di_container
            .bindings
            .insert(TypeId::of::<dyn INumber>(), Rc::new(mock_provider));

        let first_number_rc = di_container.get_singleton::<dyn INumber>()?;

        assert_eq!(first_number_rc.get(), 2820);

        let second_number_rc = di_container.get_singleton::<dyn INumber>()?;

        assert_eq!(first_number_rc.as_ref(), second_number_rc.as_ref());

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn can_get_factory() -> error_stack::Result<(), DIContainerError>
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
                ) -> error_stack::Result<Providable, ResolveError>;
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        let mut mock_provider = MockProvider::new();

        mock_provider.expect_provide().returning(|_| {
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
            .insert(TypeId::of::<IUserManagerFactory>(), Rc::new(mock_provider));

        di_container.get_factory::<IUserManagerFactory>()?;

        Ok(())
    }
}
