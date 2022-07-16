use std::marker::PhantomData;
use std::rc::Rc;

use crate::castable_factory::AnyFactory;
use crate::errors::injectable::ResolveError;
use crate::interfaces::injectable::Injectable;
use crate::DIContainer;

extern crate error_stack;

pub enum Providable
{
    Injectable(Box<dyn Injectable>),
    Factory(Rc<dyn AnyFactory>),
}

pub trait IProvider
{
    fn provide(
        &self,
        di_container: &DIContainer,
    ) -> error_stack::Result<Providable, ResolveError>;
}

pub struct InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    _phantom_data: PhantomData<InjectableType>,
}

impl<InjectableType> InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    pub fn new() -> Self
    {
        Self {
            _phantom_data: PhantomData,
        }
    }
}

impl<InjectableType> IProvider for InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    fn provide(
        &self,
        di_container: &DIContainer,
    ) -> error_stack::Result<Providable, ResolveError>
    {
        Ok(Providable::Injectable(InjectableType::resolve(
            di_container,
        )?))
    }
}

pub struct FactoryProvider
{
    _factory: Rc<dyn AnyFactory>,
}

impl FactoryProvider
{
    pub fn new(factory: Rc<dyn AnyFactory>) -> Self
    {
        Self { _factory: factory }
    }
}

impl IProvider for FactoryProvider
{
    fn provide(
        &self,
        _di_container: &DIContainer,
    ) -> error_stack::Result<Providable, ResolveError>
    {
        Ok(Providable::Factory(self._factory.clone()))
    }
}
