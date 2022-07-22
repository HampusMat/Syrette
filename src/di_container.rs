use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use error_stack::{Report, ResultExt};

#[cfg(feature = "factory")]
use crate::castable_factory::CastableFactory;
use crate::errors::di_container::DIContainerError;
use crate::interfaces::injectable::Injectable;
use crate::libs::intertrait::cast_box::CastBox;
use crate::provider::{IProvider, InjectableTypeProvider, Providable};
use crate::ptr::InterfacePtr;

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
            Rc::new(InjectableTypeProvider::<Implementation>::new()),
        );
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`DIContainer`].
    #[cfg(feature = "factory")]
    pub fn to_factory<Args, Return>(
        &mut self,
        factory_func: &'static dyn Fn<Args, Output = InterfacePtr<Return>>,
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
    bindings: HashMap<TypeId, Rc<dyn IProvider>>,
}

impl DIContainer
{
    /// Returns a new `DIContainer`.
    #[must_use]
    pub fn new() -> Self
    {
        Self {
            bindings: HashMap::new(),
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
    ) -> error_stack::Result<InterfacePtr<Interface>, DIContainerError>
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
            Providable::Injectable(binding_injectable) => {
                let interface_box_result = binding_injectable.cast::<Interface>();

                match interface_box_result {
                    Ok(interface_box) => Ok(interface_box),
                    Err(_) => Err(Report::new(DIContainerError).attach_printable(
                        format!("Unable to cast binding for {}", interface_name),
                    )),
                }
            }
            Providable::Factory(_) => Err(Report::new(DIContainerError)
                .attach_printable(format!(
                    "Binding for {} is not injectable",
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
                use crate::libs::intertrait::cast_rc::CastRc;

                let factory_box_result = binding_factory.cast::<Interface>();

                match factory_box_result {
                    Ok(interface_box) => Ok(interface_box),
                    Err(_) => Err(Report::new(DIContainerError).attach_printable(
                        format!("Unable to cast binding for {}", interface_name),
                    )),
                }
            }
            Providable::Injectable(_) => Err(Report::new(DIContainerError)
                .attach_printable(format!(
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
    use mockall::mock;

    use super::*;
    use crate::errors::injectable::ResolveError;

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
                InterfacePtr<Self>,
                crate::errors::injectable::ResolveError,
            >
            where
                Self: Sized,
            {
                Ok(InterfacePtr::new(Self {}))
            }
        }

        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.len(), 0);

        di_container.bind::<dyn IUserManager>().to::<UserManager>();

        assert_eq!(di_container.bindings.len(), 1);
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

        type IUserManagerFactory = dyn IFactory<(), dyn IUserManager>;

        let mut di_container: DIContainer = DIContainer::new();

        assert_eq!(di_container.bindings.len(), 0);

        di_container.bind::<IUserManagerFactory>().to_factory(&|| {
            let user_manager: InterfacePtr<dyn IUserManager> =
                InterfacePtr::new(UserManager::new());

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
            Ok(Providable::Injectable(
                InterfacePtr::new(UserManager::new()),
            ))
        });

        di_container
            .bindings
            .insert(TypeId::of::<dyn IUserManager>(), Rc::new(mock_provider));

        di_container.get::<dyn IUserManager>()?;

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
        type IUserManagerFactory = dyn IFactory<(Vec<i128>,), dyn IUserManager>;

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
            Ok(Providable::Factory(FactoryPtr::new(CastableFactory::new(
                &|users| {
                    let user_manager: InterfacePtr<dyn IUserManager> =
                        InterfacePtr::new(UserManager::new(users));

                    user_manager
                },
            ))))
        });

        di_container
            .bindings
            .insert(TypeId::of::<IUserManagerFactory>(), Rc::new(mock_provider));

        di_container.get_factory::<IUserManagerFactory>()?;

        Ok(())
    }
}
