use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::dependency_history::IDependencyHistory;
use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};

#[derive(strum_macros::Display, Debug)]
pub enum AsyncProvidable<DIContainerType, DependencyHistoryType>
where
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    Transient(TransientPtr<dyn AsyncInjectable<DIContainerType, DependencyHistoryType>>),
    Singleton(
        ThreadsafeSingletonPtr<
            dyn AsyncInjectable<DIContainerType, DependencyHistoryType>,
        >,
    ),
    #[cfg(feature = "factory")]
    Factory(
        crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::private::any_factory::AnyThreadsafeFactory,
        >,
    ),
    #[cfg(feature = "factory")]
    DefaultFactory(
        crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::private::any_factory::AnyThreadsafeFactory,
        >,
    ),
    #[cfg(feature = "factory")]
    AsyncDefaultFactory(
        crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::private::any_factory::AnyThreadsafeFactory,
        >,
    ),
}

#[async_trait]
#[cfg_attr(test, mockall::automock, allow(dead_code))]
pub trait IAsyncProvider<DIContainerType, DependencyHistoryType>: Send + Sync
where
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    async fn provide(
        &self,
        di_container: &Arc<DIContainerType>,
        dependency_history: DependencyHistoryType,
    ) -> Result<AsyncProvidable<DIContainerType, DependencyHistoryType>, InjectableError>;

    fn do_clone(&self)
        -> Box<dyn IAsyncProvider<DIContainerType, DependencyHistoryType>>;
}

impl<DIContainerType, DependencyHistoryType> Clone
    for Box<dyn IAsyncProvider<DIContainerType, DependencyHistoryType>>
where
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    fn clone(&self) -> Self
    {
        self.do_clone()
    }
}

pub struct AsyncTransientTypeProvider<
    InjectableType,
    DIContainerType,
    DependencyHistoryType,
> where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    injectable_phantom: PhantomData<InjectableType>,
    di_container_phantom: PhantomData<DIContainerType>,
    dependency_history_phantom: PhantomData<DependencyHistoryType>,
}

impl<InjectableType, DIContainerType, DependencyHistoryType>
    AsyncTransientTypeProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    pub fn new() -> Self
    {
        Self {
            injectable_phantom: PhantomData,
            di_container_phantom: PhantomData,
            dependency_history_phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<InjectableType, DIContainerType, DependencyHistoryType>
    IAsyncProvider<DIContainerType, DependencyHistoryType>
    for AsyncTransientTypeProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync + 'static,
{
    async fn provide(
        &self,
        di_container: &Arc<DIContainerType>,
        dependency_history: DependencyHistoryType,
    ) -> Result<AsyncProvidable<DIContainerType, DependencyHistoryType>, InjectableError>
    {
        Ok(AsyncProvidable::Transient(
            InjectableType::resolve(di_container, dependency_history).await?,
        ))
    }

    fn do_clone(&self)
        -> Box<dyn IAsyncProvider<DIContainerType, DependencyHistoryType>>
    {
        Box::new(self.clone())
    }
}

impl<InjectableType, DIContainerType, DependencyHistoryType> Clone
    for AsyncTransientTypeProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    fn clone(&self) -> Self
    {
        Self {
            injectable_phantom: self.injectable_phantom,
            di_container_phantom: PhantomData,
            dependency_history_phantom: PhantomData,
        }
    }
}

pub struct AsyncSingletonProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    singleton: ThreadsafeSingletonPtr<InjectableType>,

    di_container_phantom: PhantomData<DIContainerType>,
    dependency_history_phantom: PhantomData<DependencyHistoryType>,
}

impl<InjectableType, DIContainerType, DependencyHistoryType>
    AsyncSingletonProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    pub fn new(singleton: ThreadsafeSingletonPtr<InjectableType>) -> Self
    {
        Self {
            singleton,
            di_container_phantom: PhantomData,
            dependency_history_phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<InjectableType, DIContainerType, DependencyHistoryType>
    IAsyncProvider<DIContainerType, DependencyHistoryType>
    for AsyncSingletonProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync + 'static,
{
    async fn provide(
        &self,
        _di_container: &Arc<DIContainerType>,
        _dependency_history: DependencyHistoryType,
    ) -> Result<AsyncProvidable<DIContainerType, DependencyHistoryType>, InjectableError>
    {
        Ok(AsyncProvidable::Singleton(self.singleton.clone()))
    }

    fn do_clone(&self)
        -> Box<dyn IAsyncProvider<DIContainerType, DependencyHistoryType>>
    {
        Box::new(self.clone())
    }
}

impl<InjectableType, DIContainerType, DependencyHistoryType> Clone
    for AsyncSingletonProvider<InjectableType, DIContainerType, DependencyHistoryType>
where
    InjectableType: AsyncInjectable<DIContainerType, DependencyHistoryType>,
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync,
{
    fn clone(&self) -> Self
    {
        Self {
            singleton: self.singleton.clone(),
            di_container_phantom: PhantomData,
            dependency_history_phantom: PhantomData,
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
        dyn crate::private::any_factory::AnyThreadsafeFactory,
    >,
    variant: AsyncFactoryVariant,
}

#[cfg(feature = "factory")]
impl AsyncFactoryProvider
{
    pub fn new(
        factory: crate::ptr::ThreadsafeFactoryPtr<
            dyn crate::private::any_factory::AnyThreadsafeFactory,
        >,
        variant: AsyncFactoryVariant,
    ) -> Self
    {
        Self { factory, variant }
    }
}

#[cfg(feature = "factory")]
#[async_trait]
impl<DIContainerType, DependencyHistoryType>
    IAsyncProvider<DIContainerType, DependencyHistoryType> for AsyncFactoryProvider
where
    DIContainerType: IAsyncDIContainer<DependencyHistoryType>,
    DependencyHistoryType: IDependencyHistory + Send + Sync + 'static,
{
    async fn provide(
        &self,
        _di_container: &Arc<DIContainerType>,
        _dependency_history: DependencyHistoryType,
    ) -> Result<AsyncProvidable<DIContainerType, DependencyHistoryType>, InjectableError>
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

    fn do_clone(&self)
        -> Box<dyn IAsyncProvider<DIContainerType, DependencyHistoryType>>
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

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use super::*;
    use crate::test_utils::mocks::MockDependencyHistory;
    use crate::test_utils::{mocks, subjects_async};

    #[tokio::test]
    async fn async_transient_type_provider_works() -> Result<(), Box<dyn Error>>
    {
        let transient_type_provider = AsyncTransientTypeProvider::<
            subjects_async::UserManager,
            mocks::async_di_container::MockAsyncDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new();

        let di_container = mocks::async_di_container::MockAsyncDIContainer::new();

        assert!(
            matches!(
                transient_type_provider
                    .provide(&Arc::new(di_container), MockDependencyHistory::new())
                    .await?,
                AsyncProvidable::Transient(_)
            ),
            "The provided type is not transient"
        );

        Ok(())
    }

    #[tokio::test]
    async fn async_singleton_provider_works() -> Result<(), Box<dyn Error>>
    {
        let singleton_provider = AsyncSingletonProvider::<
            subjects_async::UserManager,
            mocks::async_di_container::MockAsyncDIContainer<mocks::MockDependencyHistory>,
            mocks::MockDependencyHistory,
        >::new(ThreadsafeSingletonPtr::new(
            subjects_async::UserManager {},
        ));

        let di_container = mocks::async_di_container::MockAsyncDIContainer::new();

        assert!(
            matches!(
                singleton_provider
                    .provide(&Arc::new(di_container), MockDependencyHistory::new())
                    .await?,
                AsyncProvidable::Singleton(_)
            ),
            "The provided type is not a singleton"
        );

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "factory")]
    async fn async_factory_provider_works() -> Result<(), Box<dyn Error>>
    {
        use crate::private::any_factory::AnyThreadsafeFactory;
        use crate::ptr::ThreadsafeFactoryPtr;

        #[derive(Debug)]
        struct FooFactory;

        impl AnyThreadsafeFactory for FooFactory {}

        let factory_provider = AsyncFactoryProvider::new(
            ThreadsafeFactoryPtr::new(FooFactory),
            AsyncFactoryVariant::Normal,
        );

        let default_factory_provider = AsyncFactoryProvider::new(
            ThreadsafeFactoryPtr::new(FooFactory),
            AsyncFactoryVariant::Default,
        );

        let async_default_factory_provider = AsyncFactoryProvider::new(
            ThreadsafeFactoryPtr::new(FooFactory),
            AsyncFactoryVariant::AsyncDefault,
        );

        let di_container =
            Arc::new(mocks::async_di_container::MockAsyncDIContainer::new());

        assert!(
            matches!(
                factory_provider
                    .provide(&di_container, mocks::MockDependencyHistory::new())
                    .await?,
                AsyncProvidable::Factory(_)
            ),
            "The provided type is not a factory"
        );

        assert!(
            matches!(
                default_factory_provider
                    .provide(&di_container, MockDependencyHistory::new())
                    .await?,
                AsyncProvidable::DefaultFactory(_)
            ),
            "The provided type is not a default factory"
        );

        assert!(
            matches!(
                async_default_factory_provider
                    .provide(&di_container, MockDependencyHistory::new())
                    .await?,
                AsyncProvidable::AsyncDefaultFactory(_)
            ),
            "The provided type is not a async default factory"
        );

        Ok(())
    }
}
