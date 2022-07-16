use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use error_stack::{Report, ResultExt};

use crate::castable_factory::CastableFactory;
use crate::errors::di_container::DIContainerError;
use crate::interfaces::factory::IFactory;
use crate::interfaces::injectable::Injectable;
use crate::libs::intertrait::cast_box::CastBox;
use crate::libs::intertrait::cast_rc::CastRc;
use crate::provider::{FactoryProvider, IProvider, InjectableTypeProvider, Providable};

/// Binding builder for type `Interface` inside a [`DIContainer`].
pub struct BindingBuilder<'a, Interface>
where
    Interface: 'static + ?Sized,
{
    _di_container: &'a mut DIContainer,
    _phantom_data: PhantomData<Interface>,
}

impl<'a, Interface> BindingBuilder<'a, Interface>
where
    Interface: 'static + ?Sized,
{
    fn new(di_container: &'a mut DIContainer) -> Self
    {
        Self {
            _di_container: di_container,
            _phantom_data: PhantomData,
        }
    }

    /// Creates a binding of type `Interface` to type `Implementation` inside of the
    /// associated [`DIContainer`].
    pub fn to<Implementation>(&mut self)
    where
        Implementation: Injectable,
    {
        let interface_typeid = TypeId::of::<Interface>();

        self._di_container._bindings.insert(
            interface_typeid,
            Rc::new(InjectableTypeProvider::<Implementation>::new()),
        );
    }

    /// Creates a binding of factory type `Interface` to a factory inside of the
    /// associated [`DIContainer`].
    pub fn to_factory<Args, Return>(
        &mut self,
        factory_func: &'static dyn Fn<Args, Output = Box<Return>>,
    ) where
        Args: 'static,
        Return: 'static + ?Sized,
        Interface: IFactory<Args, Return>,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let factory_impl = CastableFactory::new(factory_func);

        self._di_container._bindings.insert(
            interface_typeid,
            Rc::new(FactoryProvider::new(Rc::new(factory_impl))),
        );
    }
}

/// Dependency injection container.
///
/// # Examples
/// ```
/// di_container.bind::<dyn IDatabaseService>().to::<DatabaseService>();
///
/// let database_service = di_container.get::<dyn IDatabaseService>()?;
/// ```
pub struct DIContainer
{
    _bindings: HashMap<TypeId, Rc<dyn IProvider>>,
}

impl<'a> DIContainer
{
    /// Returns a new `DIContainer`.
    pub fn new() -> Self
    {
        Self {
            _bindings: HashMap::new(),
        }
    }

    /// Returns a new [`BindingBuilder`] for the given interface.
    pub fn bind<Interface>(&'a mut self) -> BindingBuilder<Interface>
    where
        Interface: 'static + ?Sized,
    {
        BindingBuilder::<Interface>::new(self)
    }

    /// Returns a new instance of the type bound with `Interface`.
    pub fn get<Interface>(&self) -> error_stack::Result<Box<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let interface_name = type_name::<Interface>();

        let binding = self._bindings.get(&interface_typeid).ok_or_else(|| {
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
    pub fn get_factory<Interface>(
        &self,
    ) -> error_stack::Result<Rc<Interface>, DIContainerError>
    where
        Interface: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<Interface>();

        let interface_name = type_name::<Interface>();

        let binding = self._bindings.get(&interface_typeid).ok_or_else(|| {
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
