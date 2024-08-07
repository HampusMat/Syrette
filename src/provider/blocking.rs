use std::marker::PhantomData;

use crate::errors::injectable::InjectableError;
use crate::interfaces::injectable::Injectable;
use crate::ptr::{SingletonPtr, TransientPtr};
use crate::util::use_double;

use_double!(crate::dependency_history::DependencyHistory);

#[derive(strum_macros::Display, Debug)]
pub enum Providable<DIContainerType>
{
    Transient(TransientPtr<dyn Injectable<DIContainerType>>),
    Singleton(SingletonPtr<dyn Injectable<DIContainerType>>),
    #[cfg(feature = "factory")]
    Factory(crate::ptr::FactoryPtr<dyn crate::castable_function::AnyCastableFunction>),
    #[cfg(feature = "factory")]
    DefaultFactory(
        crate::ptr::FactoryPtr<dyn crate::castable_function::AnyCastableFunction>,
    ),
}

#[cfg_attr(test, mockall::automock)]
pub trait IProvider<DIContainerType>
{
    fn provide(
        &self,
        di_container: &DIContainerType,
        dependency_history: DependencyHistory,
    ) -> Result<Providable<DIContainerType>, InjectableError>;
}

pub struct TransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
{
    injectable_phantom: PhantomData<InjectableType>,
    di_container_phantom: PhantomData<DIContainerType>,
}

impl<InjectableType, DIContainerType>
    TransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
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
{
    fn provide(
        &self,
        di_container: &DIContainerType,
        dependency_history: DependencyHistory,
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
{
    singleton: SingletonPtr<InjectableType>,

    di_container_phantom: PhantomData<DIContainerType>,
}

impl<InjectableType, DIContainerType> SingletonProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
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
{
    fn provide(
        &self,
        _di_container: &DIContainerType,
        _dependency_history: DependencyHistory,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(Providable::Singleton(self.singleton.clone()))
    }
}

#[cfg(feature = "factory")]
pub struct FactoryProvider
{
    factory: crate::ptr::FactoryPtr<dyn crate::castable_function::AnyCastableFunction>,
    is_default_factory: bool,
}

#[cfg(feature = "factory")]
impl FactoryProvider
{
    pub fn new(
        factory: crate::ptr::FactoryPtr<
            dyn crate::castable_function::AnyCastableFunction,
        >,
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
{
    fn provide(
        &self,
        _di_container: &DIContainerType,
        _dependency_history: DependencyHistory,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(if self.is_default_factory {
            Providable::DefaultFactory(self.factory.clone())
        } else {
            Providable::Factory(self.factory.clone())
        })
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::di_container::blocking::MockDIContainer;
    use crate::test_utils::subjects;

    #[test]
    fn transient_type_provider_works()
    {
        let transient_type_provider =
            TransientTypeProvider::<subjects::UserManager, MockDIContainer>::new();

        let di_container = MockDIContainer::new();

        let dependency_history_mock = MockDependencyHistory::new();

        assert!(
            matches!(
                transient_type_provider.provide(&di_container, dependency_history_mock),
                Ok(Providable::Transient(_))
            ),
            "The provided type is not transient"
        );
    }

    #[test]
    fn singleton_provider_works()
    {
        let singleton_provider =
            SingletonProvider::<subjects::UserManager, MockDIContainer>::new(
                SingletonPtr::new(subjects::UserManager {}),
            );

        let di_container = MockDIContainer::new();

        assert!(
            matches!(
                singleton_provider
                    .provide(&di_container, MockDependencyHistory::new())
                    .unwrap(),
                Providable::Singleton(_)
            ),
            "The provided type is not a singleton"
        );
    }

    #[test]
    #[cfg(feature = "factory")]
    fn factory_provider_works()
    {
        use std::any::Any;

        use crate::castable_function::AnyCastableFunction;
        use crate::ptr::FactoryPtr;

        #[derive(Debug)]
        struct FooFactory;

        impl AnyCastableFunction for FooFactory
        {
            fn as_any(&self) -> &dyn Any
            {
                self
            }
        }

        let factory_provider = FactoryProvider::new(FactoryPtr::new(FooFactory), false);
        let default_factory_provider =
            FactoryProvider::new(FactoryPtr::new(FooFactory), true);

        let di_container = MockDIContainer::new();

        assert!(
            matches!(
                factory_provider.provide(&di_container, MockDependencyHistory::new()),
                Ok(Providable::Factory(_))
            ),
            "The provided type is not a factory"
        );

        assert!(
            matches!(
                default_factory_provider
                    .provide(&di_container, MockDependencyHistory::new()),
                Ok(Providable::DefaultFactory(_))
            ),
            "The provided type is not a default factory"
        );
    }
}
