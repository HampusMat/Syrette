use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::di_container::asynchronous::IAsyncDIContainer;
use crate::errors::injectable::InjectableError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};
use crate::util::use_dependency_history;

use_dependency_history!();

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
pub trait IAsyncProvider<DIContainerType>: Send + Sync
where
    DIContainerType: IAsyncDIContainer,
{
    async fn provide(
        &self,
        di_container: &Arc<DIContainerType>,
        dependency_history: DependencyHistory,
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
        dependency_history: DependencyHistory,
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
            di_container_phantom: PhantomData,
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
        _dependency_history: DependencyHistory,
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
impl<DIContainerType> IAsyncProvider<DIContainerType> for AsyncFactoryProvider
where
    DIContainerType: IAsyncDIContainer,
{
    async fn provide(
        &self,
        _di_container: &Arc<DIContainerType>,
        _dependency_history: DependencyHistory,
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

#[cfg(test)]
mod tests
{
    use std::error::Error;

    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::test_utils::{mocks, subjects_async};

    #[tokio::test]
    async fn async_transient_type_provider_works() -> Result<(), Box<dyn Error>>
    {
        let transient_type_provider = AsyncTransientTypeProvider::<
            subjects_async::UserManager,
            mocks::async_di_container::MockAsyncDIContainer,
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
            mocks::async_di_container::MockAsyncDIContainer,
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
                    .provide(&di_container, MockDependencyHistory::new())
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
