use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};

#[derive(strum_macros::Display, Debug)]
pub enum AsyncProvidable<DIContainerType>
where
    DIContainerType: IAsyncDIContainer,
{
    Transient(TransientPtr<dyn AsyncInjectable<DIContainerType>>),
    Singleton(ThreadsafeSingletonPtr<dyn AsyncInjectable<DIContainerType>>),
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
pub trait IAsyncProvider<DIContainerType>: Send + Sync
where
    DIContainerType: IAsyncDIContainer,
{
    async fn provide(
        &self,
        di_container: &Arc<DIContainerType>,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable<DIContainerType>, InjectableError>;

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerType>>;
}

impl<DIContainerType> Clone for Box<dyn IAsyncProvider<DIContainerType>>
where
    DIContainerType: IAsyncDIContainer,
{
    fn clone(&self) -> Self
    {
        self.do_clone()
    }
}

pub struct AsyncTransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    injectable_phantom: PhantomData<InjectableType>,
    di_container_phantom: PhantomData<DIContainerType>,
}

impl<InjectableType, DIContainerType>
    AsyncTransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    pub fn new() -> Self
    {
        Self {
            injectable_phantom: PhantomData,
            di_container_phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<InjectableType, DIContainerType> IAsyncProvider<DIContainerType>
    for AsyncTransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    async fn provide(
        &self,
        di_container: &Arc<DIContainerType>,
        dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable<DIContainerType>, InjectableError>
    {
        Ok(AsyncProvidable::Transient(
            InjectableType::resolve(di_container, dependency_history).await?,
        ))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerType>>
    {
        Box::new(self.clone())
    }
}

impl<InjectableType, DIContainerType> Clone
    for AsyncTransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    fn clone(&self) -> Self
    {
        Self {
            injectable_phantom: self.injectable_phantom,
            di_container_phantom: self.di_container_phantom,
        }
    }
}

pub struct AsyncSingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    singleton: ThreadsafeSingletonPtr<InjectableType>,

    di_container_phantom: PhantomData<DIContainerType>,
}

impl<InjectableType, DIContainerType>
    AsyncSingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    pub fn new(singleton: ThreadsafeSingletonPtr<InjectableType>) -> Self
    {
        Self {
            singleton,
            di_container_phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<InjectableType, DIContainerType> IAsyncProvider<DIContainerType>
    for AsyncSingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    async fn provide(
        &self,
        _di_container: &Arc<DIContainerType>,
        _dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable<DIContainerType>, InjectableError>
    {
        Ok(AsyncProvidable::Singleton(self.singleton.clone()))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerType>>
    {
        Box::new(self.clone())
    }
}

impl<InjectableType, DIContainerType> Clone
    for AsyncSingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: AsyncInjectable<DIContainerType>,
    DIContainerType: IAsyncDIContainer,
{
    fn clone(&self) -> Self
    {
        Self {
            singleton: self.singleton.clone(),
            di_container_phantom: PhantomData,
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
impl<DIContainerType> IAsyncProvider<DIContainerType> for AsyncFactoryProvider
where
    DIContainerType: IAsyncDIContainer,
{
    async fn provide(
        &self,
        _di_container: &Arc<DIContainerType>,
        _dependency_history: Vec<&'static str>,
    ) -> Result<AsyncProvidable<DIContainerType>, InjectableError>
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

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerType>>
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
