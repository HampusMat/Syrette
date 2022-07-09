use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

use error_stack::{Context, Report, ResultExt};

use crate::injectable::Injectable;
use crate::libs::intertrait::cast_box::CastBox;
use crate::provider::{IInjectableTypeProvider, InjectableTypeProvider};

/// Binding builder for `InterfaceTrait` in a [`DIContainer`].
pub struct BindingBuilder<'a, InterfaceTrait>
where
    InterfaceTrait: 'static + ?Sized,
{
    _di_container: &'a mut DIContainer,
    _phantom_data: PhantomData<InterfaceTrait>,
}

impl<'a, InterfaceTrait> BindingBuilder<'a, InterfaceTrait>
where
    InterfaceTrait: 'static + ?Sized,
{
    fn new(di_container: &'a mut DIContainer) -> Self
    {
        Self {
            _di_container: di_container,
            _phantom_data: PhantomData,
        }
    }

    /// Creates a binding of `InterfaceTrait` to type `Implementation` inside of the
    /// associated [`DIContainer`].
    pub fn to<Implementation>(&mut self)
    where
        Implementation: Injectable,
    {
        let interface_typeid = TypeId::of::<InterfaceTrait>();

        self._di_container._bindings.insert(
            interface_typeid,
            Rc::new(InjectableTypeProvider::<Implementation>::new()),
        );
    }
}

#[derive(Debug)]
pub struct DIContainerError;

impl Display for DIContainerError
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result
    {
        fmt.write_str("A DI container error has occurred")
    }
}

impl Context for DIContainerError {}

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
    _bindings: HashMap<TypeId, Rc<dyn IInjectableTypeProvider>>,
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

    /// Returns a new [`BindingBuilder`] for the given interface trait.
    pub fn bind<InterfaceTrait>(&'a mut self) -> BindingBuilder<InterfaceTrait>
    where
        InterfaceTrait: 'static + ?Sized,
    {
        BindingBuilder::<InterfaceTrait>::new(self)
    }

    /// Returns the value bound with `InterfaceTrait`.
    pub fn get<InterfaceTrait>(
        &self,
    ) -> error_stack::Result<Box<InterfaceTrait>, DIContainerError>
    where
        InterfaceTrait: 'static + ?Sized,
    {
        let interface_typeid = TypeId::of::<InterfaceTrait>();

        let interface_name = type_name::<InterfaceTrait>();

        let binding = self._bindings.get(&interface_typeid).ok_or_else(|| {
            Report::new(DIContainerError)
                .attach_printable(format!("No binding exists for {}", interface_name))
        })?;

        let binding_injectable = binding
            .provide(self)
            .change_context(DIContainerError)
            .attach_printable(format!(
            "Failed to resolve interface {}",
            interface_name
        ))?;

        let interface_box_result = binding_injectable.cast::<InterfaceTrait>();

        match interface_box_result {
            Ok(interface_box) => Ok(interface_box),
            Err(_) => Err(Report::new(DIContainerError).attach_printable(format!(
                "Unable to cast binding for {}",
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
