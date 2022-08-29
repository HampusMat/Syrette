#![allow(clippy::module_name_repetitions)]
use std::marker::PhantomData;

use async_trait::async_trait;

use crate::async_di_container::AsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};

#[derive(strum_macros::Display, Debug)]
pub enum AsyncProvidable
{
    Transient(TransientPtr<dyn AsyncInjectable>),
    Singleton(ThreadsafeSingletonPtr<dyn AsyncInjectable>),
    #[cfg(feature = "factory")]
    Factory(
        crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::interfaces::any_factory::AnyThreadsafeFactory,
        >,
    ),
}

#[async_trait]
pub trait IAsyncProvider: Send + Sync
{
    async fn provide(
        &self,
        di_container: &AsyncDIContainer,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>;
}

pub struct AsyncTransientTypeProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    injectable_phantom: PhantomData<InjectableType>,
}

impl<InjectableType> AsyncTransientTypeProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    pub fn new() -> Self
    {
        Self {
            injectable_phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<InjectableType> IAsyncProvider for AsyncTransientTypeProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    async fn provide(
        &self,
        di_container: &AsyncDIContainer,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>
    {
        Ok(AsyncProvidable::Transient(
            InjectableType::resolve(di_container, dependency_history).await?,
        ))
    }
}

pub struct AsyncSingletonProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    singleton: ThreadsafeSingletonPtr<InjectableType>,
}

impl<InjectableType> AsyncSingletonProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    pub fn new(singleton: ThreadsafeSingletonPtr<InjectableType>) -> Self
    {
        Self { singleton }
    }
}

#[async_trait]
impl<InjectableType> IAsyncProvider for AsyncSingletonProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    async fn provide(
        &self,
        _di_container: &AsyncDIContainer,
        _dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>
    {
        Ok(AsyncProvidable::Singleton(self.singleton.clone()))
    }
}

#[cfg(feature = "factory")]
pub struct AsyncFactoryProvider
{
    factory: crate::ptr::ThreadsafeFactoryPtr<
        dyn crate::interfaces::any_factory::AnyThreadsafeFactory,
    >,
}

#[cfg(feature = "factory")]
impl AsyncFactoryProvider
{
    pub fn new(
        factory: crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::interfaces::any_factory::AnyThreadsafeFactory,
        >,
    ) -> Self
    {
        Self { factory }
    }
}

#[cfg(feature = "factory")]
#[async_trait]
impl IAsyncProvider for AsyncFactoryProvider
{
    async fn provide(
        &self,
        _di_container: &AsyncDIContainer,
        _dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>
    {
        Ok(AsyncProvidable::Factory(self.factory.clone()))
    }
}
