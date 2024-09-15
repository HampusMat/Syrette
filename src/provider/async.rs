use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::castable_function::threadsafe::AnyThreadsafeCastableFunction;
use crate::errors::injectable::InjectableError;
use crate::interfaces::async_injectable::AsyncInjectable;
use crate::ptr::{ThreadsafeSingletonPtr, TransientPtr};
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

#[derive(strum_macros::Display, Debug)]
pub enum AsyncProvidable<DIContainerT>
{
    Transient(TransientPtr<dyn AsyncInjectable<DIContainerT>>),
    Singleton(ThreadsafeSingletonPtr<dyn AsyncInjectable<DIContainerT>>),
    Function(
        Arc<dyn crate::castable_function::threadsafe::AnyThreadsafeCastableFunction>,
        ProvidableFunctionKind,
    ),
}

#[derive(Debug, Clone, Copy)]
pub enum ProvidableFunctionKind
{
    #[cfg(feature = "factory")]
    UserCalled,
    Instant,
    AsyncInstant,
}

#[async_trait]
#[cfg_attr(test, mockall::automock, allow(dead_code))]
pub trait IAsyncProvider<DIContainerT>: Send + Sync
where
    DIContainerT: Send + Sync,
{
    async fn provide(
        &self,
        di_container: &DIContainerT,
        dependency_history: DependencyHistory,
    ) -> Result<AsyncProvidable<DIContainerT>, InjectableError>;

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerT>>;
}

impl<DIContainerT> Clone for Box<dyn IAsyncProvider<DIContainerT>>
where
    DIContainerT: Send + Sync,
{
    fn clone(&self) -> Self
    {
        self.do_clone()
    }
}

pub struct AsyncTransientTypeProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
{
    injectable_phantom: PhantomData<InjectableT>,
    di_container_phantom: PhantomData<DIContainerT>,
}

impl<InjectableT, DIContainerT> AsyncTransientTypeProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
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
impl<InjectableT, DIContainerT> IAsyncProvider<DIContainerT>
    for AsyncTransientTypeProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
    DIContainerT: Send + Sync + 'static,
{
    async fn provide(
        &self,
        di_container: &DIContainerT,
        dependency_history: DependencyHistory,
    ) -> Result<AsyncProvidable<DIContainerT>, InjectableError>
    {
        Ok(AsyncProvidable::Transient(
            InjectableT::resolve(di_container, dependency_history).await?,
        ))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerT>>
    {
        Box::new(self.clone())
    }
}

impl<InjectableT, DIContainerT> Clone
    for AsyncTransientTypeProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
{
    fn clone(&self) -> Self
    {
        Self {
            injectable_phantom: self.injectable_phantom,
            di_container_phantom: PhantomData,
        }
    }
}

pub struct AsyncSingletonProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
{
    singleton: ThreadsafeSingletonPtr<InjectableT>,

    di_container_phantom: PhantomData<DIContainerT>,
}

impl<InjectableT, DIContainerT> AsyncSingletonProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
{
    pub fn new(singleton: ThreadsafeSingletonPtr<InjectableT>) -> Self
    {
        Self {
            singleton,
            di_container_phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<InjectableT, DIContainerT> IAsyncProvider<DIContainerT>
    for AsyncSingletonProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
    DIContainerT: Send + Sync + 'static,
{
    async fn provide(
        &self,
        _di_container: &DIContainerT,
        _dependency_history: DependencyHistory,
    ) -> Result<AsyncProvidable<DIContainerT>, InjectableError>
    {
        Ok(AsyncProvidable::Singleton(self.singleton.clone()))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerT>>
    {
        Box::new(self.clone())
    }
}

impl<InjectableT, DIContainerT> Clone
    for AsyncSingletonProvider<InjectableT, DIContainerT>
where
    InjectableT: AsyncInjectable<DIContainerT>,
{
    fn clone(&self) -> Self
    {
        Self {
            singleton: self.singleton.clone(),
            di_container_phantom: PhantomData,
        }
    }
}

pub struct AsyncFunctionProvider
{
    function: Arc<dyn AnyThreadsafeCastableFunction>,
    providable_func_kind: ProvidableFunctionKind,
}

impl AsyncFunctionProvider
{
    pub fn new(
        function: Arc<dyn AnyThreadsafeCastableFunction>,
        providable_func_kind: ProvidableFunctionKind,
    ) -> Self
    {
        Self {
            function,
            providable_func_kind,
        }
    }
}

#[async_trait]
impl<DIContainerT> IAsyncProvider<DIContainerT> for AsyncFunctionProvider
where
    DIContainerT: Send + Sync,
{
    async fn provide(
        &self,
        _di_container: &DIContainerT,
        _dependency_history: DependencyHistory,
    ) -> Result<AsyncProvidable<DIContainerT>, InjectableError>
    {
        Ok(AsyncProvidable::Function(
            self.function.clone(),
            self.providable_func_kind,
        ))
    }

    fn do_clone(&self) -> Box<dyn IAsyncProvider<DIContainerT>>
    {
        Box::new(self.clone())
    }
}

impl Clone for AsyncFunctionProvider
{
    fn clone(&self) -> Self
    {
        Self {
            function: self.function.clone(),
            providable_func_kind: self.providable_func_kind,
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::di_container::asynchronous::MockAsyncDIContainer;
    use crate::test_utils::subjects_async;

    #[tokio::test]
    async fn async_transient_type_provider_works()
    {
        let transient_type_provider = AsyncTransientTypeProvider::<
            subjects_async::UserManager,
            MockAsyncDIContainer,
        >::new();

        let di_container = MockAsyncDIContainer::new();

        assert!(
            matches!(
                transient_type_provider
                    .provide(&di_container, MockDependencyHistory::new())
                    .await
                    .unwrap(),
                AsyncProvidable::Transient(_)
            ),
            "The provided type is not transient"
        );
    }

    #[tokio::test]
    async fn async_singleton_provider_works()
    {
        let singleton_provider = AsyncSingletonProvider::<
            subjects_async::UserManager,
            MockAsyncDIContainer,
        >::new(ThreadsafeSingletonPtr::new(
            subjects_async::UserManager {},
        ));

        let di_container = MockAsyncDIContainer::new();

        assert!(
            matches!(
                singleton_provider
                    .provide(&di_container, MockDependencyHistory::new())
                    .await
                    .unwrap(),
                AsyncProvidable::Singleton(_)
            ),
            "The provided type is not a singleton"
        );
    }

    #[tokio::test]
    async fn function_provider_works()
    {
        use std::any::Any;
        use std::sync::Arc;

        use crate::castable_function::threadsafe::AnyThreadsafeCastableFunction;
        use crate::castable_function::AnyCastableFunction;

        #[derive(Debug)]
        struct FooFactory;

        impl AnyCastableFunction for FooFactory
        {
            fn as_any(&self) -> &dyn Any
            {
                self
            }
        }

        impl AnyThreadsafeCastableFunction for FooFactory {}

        let instant_func_provider = AsyncFunctionProvider::new(
            Arc::new(FooFactory),
            ProvidableFunctionKind::Instant,
        );

        let di_container = MockAsyncDIContainer::new();

        assert!(
            matches!(
                instant_func_provider
                    .provide(&di_container, MockDependencyHistory::new())
                    .await
                    .unwrap(),
                AsyncProvidable::Function(_, ProvidableFunctionKind::Instant)
            ),
            concat!(
                "The provided type is not a AsyncProvidable::Function of kind ",
                "ProvidableFunctionKind::Instant"
            )
        );
    }
}
