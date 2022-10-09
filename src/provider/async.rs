use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::errors::injectable::InjectableError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};
use crate::AsyncDIContainer;

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
    #[cfg(feature = "factory")]
    DefaultFactory(
        crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::interfaces::any_factory::AnyThreadsafeFactory,
        >,
    ),
    #[cfg(feature = "factory")]
    AsyncDefaultFactory(
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
        di_container: &Arc<AsyncDIContainer>,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>;

    fn do_clone(&self) -> Box<dyn IAsyncProvider>;
}

impl Clone for Box<dyn IAsyncProvider>
{
    fn clone(&self) -> Self
    {
        self.do_clone()
    }
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
        di_container: &Arc<AsyncDIContainer>,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>
    {
        Ok(AsyncProvidable::Transient(
            InjectableType::resolve(di_container, dependency_history).await?,
        ))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider>
    {
        Box::new(self.clone())
    }
}

impl<InjectableType> Clone for AsyncTransientTypeProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    fn clone(&self) -> Self
    {
        Self {
            injectable_phantom: self.injectable_phantom,
        }
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
        _di_container: &Arc<AsyncDIContainer>,
        _dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>
    {
        Ok(AsyncProvidable::Singleton(self.singleton.clone()))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider>
    {
        Box::new(self.clone())
    }
}

impl<InjectableType> Clone for AsyncSingletonProvider<InjectableType>
where
    InjectableType: AsyncInjectable,
{
    fn clone(&self) -> Self
    {
        Self {
            singleton: self.singleton.clone(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum AsyncFactoryVariant
{
    Normal,
    Default,
    AsyncDefault,
}

#[cfg(feature = "factory")]
pub struct AsyncFactoryProvider
{
    factory: crate::ptr::ThreadsafeFactoryPtr<
        dyn crate::interfaces::any_factory::AnyThreadsafeFactory,
    >,
    variant: AsyncFactoryVariant,
}

#[cfg(feature = "factory")]
impl AsyncFactoryProvider
{
    pub fn new(
        factory: crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::interfaces::any_factory::AnyThreadsafeFactory,
        >,
        variant: AsyncFactoryVariant,
    ) -> Self
    {
        Self { factory, variant }
    }
}

#[cfg(feature = "factory")]
#[async_trait]
impl IAsyncProvider for AsyncFactoryProvider
{
    async fn provide(
        &self,
        _di_container: &Arc<AsyncDIContainer>,
        _dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable, InjectableError>
    {
        Ok(match self.variant {
            AsyncFactoryVariant::Normal => AsyncProvidable::Factory(self.factory.clone()),
            AsyncFactoryVariant::Default => {
                AsyncProvidable::DefaultFactory(self.factory.clone())
            }
            AsyncFactoryVariant::AsyncDefault => {
                AsyncProvidable::AsyncDefaultFactory(self.factory.clone())
            }
        })
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider>
    {
        Box::new(self.clone())
    }
}

#[cfg(feature = "factory")]
impl Clone for AsyncFactoryProvider
{
    fn clone(&self) -> Self
    {
        Self {
            factory: self.factory.clone(),
            variant: self.variant,
        }
    }
}
