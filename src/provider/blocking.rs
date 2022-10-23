use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::IDIContainer;
use crate::errors::injectable::InjectableError;
use crate::interfaces::injectable::Injectable;
use crate::ptr::{SingletonPtr, TransientPtr};

#[derive(strum_macros::Display, Debug)]
pub enum Providable<DIContainerType>
where
    DIContainerType: IDIContainer,
{
    Transient(TransientPtr<dyn Injectable<DIContainerType>>),
    Singleton(SingletonPtr<dyn Injectable<DIContainerType>>),
    #[cfg(feature = "factory")]
    Factory(crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>),
    #[cfg(feature = "factory")]
    DefaultFactory(
        crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>,
    ),
}

pub trait IProvider<DIContainerType>
where
    DIContainerType: IDIContainer,
{
    fn provide(
        &self,
        di_container: &Rc<DIContainerType>,
        dependency_history: Vec<&'static str>,
    ) -> Result<Providable<DIContainerType>, InjectableError>;
}

pub struct TransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    injectable_phantom: PhantomData<InjectableType>,
    di_container_phantom: PhantomData<DIContainerType>,
}

impl<InjectableType, DIContainerType>
    TransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    pub fn new() -> Self
    {
        Self {
            injectable_phantom: PhantomData,
            di_container_phantom: PhantomData,
        }
    }
}

impl<InjectableType, DIContainerType> IProvider<DIContainerType>
    for TransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    fn provide(
        &self,
        di_container: &Rc<DIContainerType>,
        dependency_history: Vec<&'static str>,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(Providable::Transient(InjectableType::resolve(
            di_container,
            dependency_history,
        )?))
    }
}

pub struct SingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    singleton: SingletonPtr<InjectableType>,
    di_container_phantom: PhantomData<DIContainerType>,
}

impl<InjectableType, DIContainerType> SingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    pub fn new(singleton: SingletonPtr<InjectableType>) -> Self
    {
        Self {
            singleton,
            di_container_phantom: PhantomData,
        }
    }
}

impl<InjectableType, DIContainerType> IProvider<DIContainerType>
    for SingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    fn provide(
        &self,
        _di_container: &Rc<DIContainerType>,
        _dependency_history: Vec<&'static str>,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(Providable::Singleton(self.singleton.clone()))
    }
}

#[cfg(feature = "factory")]
pub struct FactoryProvider
{
    factory: crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>,
    is_default_factory: bool,
}

#[cfg(feature = "factory")]
impl FactoryProvider
{
    pub fn new(
        factory: crate::ptr::FactoryPtr<dyn crate::interfaces::any_factory::AnyFactory>,
        is_default_factory: bool,
    ) -> Self
    {
        Self {
            factory,
            is_default_factory,
        }
    }
}

#[cfg(feature = "factory")]
impl<DIContainerType> IProvider<DIContainerType> for FactoryProvider
where
    DIContainerType: IDIContainer,
{
    fn provide(
        &self,
        _di_container: &Rc<DIContainerType>,
        _dependency_history: Vec<&'static str>,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(if self.is_default_factory {
            Providable::DefaultFactory(self.factory.clone())
        } else {
            Providable::Factory(self.factory.clone())
        })
    }
}
