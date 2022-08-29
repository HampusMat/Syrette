#![allow(clippy::module_name_repetitions)]
use std::marker::PhantomData;

use crate::errors::injectable::InjectableError;
use crate::interfaces::injectable::Injectable;
use crate::ptr::{SingletonPtr, TransientPtr};
use crate::DIContainer;

#[derive(strum_macros::Display, Debug)]
pub enum Providable
{
    Transient(TransientPtr<dyn Injectable>),
    Singleton(SingletonPtr<dyn Injectable>),
    #[cfg(feature = "factory")]
    Factory(crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>),
}

pub trait IProvider
{
    fn provide(
        &self,
        di_container: &DIContainer,
        dependency_history: Vec<&'static str>,
    ) -> Result<Providable, InjectableError>;
}

pub struct TransientTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    injectable_phantom: PhantomData<InjectableType>,
}

impl<InjectableType> TransientTypeProvider<InjectableType>
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

impl<InjectableType> IProvider for TransientTypeProvider<InjectableType>
where
    InjectableType: Injectable,
{
    fn provide(
        &self,
        di_container: &DIContainer,
        dependency_history: Vec<&'static str>,
    ) -> Result<Providable, InjectableError>
    {
        Ok(Providable::Transient(InjectableType::resolve(
            di_container,
            dependency_history,
        )?))
    }
}

pub struct SingletonProvider<InjectableType>
where
    InjectableType: Injectable,
{
    singleton: SingletonPtr<InjectableType>,
}

impl<InjectableType> SingletonProvider<InjectableType>
where
    InjectableType: Injectable,
{
    pub fn new(singleton: SingletonPtr<InjectableType>) -> Self
    {
        Self { singleton }
    }
}

impl<InjectableType> IProvider for SingletonProvider<InjectableType>
where
    InjectableType: Injectable,
{
    fn provide(
        &self,
        _di_container: &DIContainer,
        _dependency_history: Vec<&'static str>,
    ) -> Result<Providable, InjectableError>
    {
        Ok(Providable::Singleton(self.singleton.clone()))
    }
}

#[cfg(feature = "factory")]
pub struct FactoryProvider
{
    factory: crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>,
}

#[cfg(feature = "factory")]
impl FactoryProvider
{
    pub fn new(
        factory: crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>,
    ) -> Self
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
        _dependency_history: Vec<&'static str>,
    ) -> Result<Providable, InjectableError>
    {
        Ok(Providable::Factory(self.factory.clone()))
    }
}
