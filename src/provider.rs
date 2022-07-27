#![allow(clippy::module_name_repetitions)]
use std::marker::PhantomData;

use crate::errors::injectable::ResolveError;
use crate::interfaces::any_factory::AnyFactory;
use crate::interfaces::injectable::Injectable;
use crate::ptr::{FactoryPtr, TransientPtr};
use crate::DIContainer;

extern crate error_stack;

pub enum Providable
{
    Injectable(TransientPtr<dyn Injectable>),
    #[allow(dead_code)]
    Factory(FactoryPtr<dyn AnyFactory>),
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
    injectable_phantom: PhantomData<InjectableType>,
}

impl<InjectableType> InjectableTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    pub fn new() -> Self
    {
        Self {
            injectable_phantom: PhantomData,
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

#[cfg(feature = "factory")]
pub struct FactoryProvider
{
    factory: FactoryPtr<dyn AnyFactory>,
}

#[cfg(feature = "factory")]
impl FactoryProvider
{
    pub fn new(factory: FactoryPtr<dyn AnyFactory>) -> Self
    {
        Self { factory }
    }
}

#[cfg(feature = "factory")]
impl IProvider for FactoryProvider
{
    fn provide(
        &self,
        _di_container: &DIContainer,
    ) -> error_stack::Result<Providable, ResolveError>
    {
        Ok(Providable::Factory(self.factory.clone()))
    }
}
