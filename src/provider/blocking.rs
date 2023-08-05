use std::marker::PhantomData;
use std::rc::Rc;

use crate::di_container::blocking::IDIContainer;
use crate::errors::injectable::InjectableError;
use crate::interfaces::injectable::Injectable;
use crate::ptr::{SingletonPtr, TransientPtr};
use crate::util::use_dependency_history;

use_dependency_history!();

#[derive(strum_macros::Display, Debug)]
pub enum Providable<DIContainerType>
where
    DIContainerType: IDIContainer,
{
    Transient(TransientPtr<dyn Injectable<DIContainerType>>),
    Singleton(SingletonPtr<dyn Injectable<DIContainerType>>),
    #[cfg(feature = "factory")]
    Factory(crate::ptr::FactoryPtr<dyn crate::private::any_factory::AnyFactory>),
    #[cfg(feature = "factory")]
    DefaultFactory(crate::ptr::FactoryPtr<dyn crate::private::any_factory::AnyFactory>),
}

#[cfg_attr(test, mockall::automock, allow(dead_code))]
pub trait IProvider<DIContainerType>
where
    DIContainerType: IDIContainer,
{
    fn provide(
        &self,
        di_container: &Rc<DIContainerType>,
        dependency_history: DependencyHistory,
    ) -> Result<Providable<DIContainerType>, InjectableError>;
}

pub struct TransientTypeProvider<InjectableType, DIContainerType>
where
    InjectableType: Injectable<DIContainerType>,
    DIContainerType: IDIContainer,
{
    injectable_phantom: PhantomData<InjectableType>,
    di_container_phantom: PhantomData<DIContainerType>,
    dependency_history_phantom: PhantomData<DependencyHistory>,
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
            dependency_history_phantom: PhantomData,
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
    DIContainerType: IDIContainer,
{
    singleton: SingletonPtr<InjectableType>,

    di_container_phantom: PhantomData<DIContainerType>,
    dependency_history_phantom: PhantomData<DependencyHistory>,
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
            dependency_history_phantom: PhantomData,
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
        _dependency_history: DependencyHistory,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(Providable::Singleton(self.singleton.clone()))
    }
}

#[cfg(feature = "factory")]
pub struct FactoryProvider
{
    factory: crate::ptr::FactoryPtr<dyn crate::private::any_factory::AnyFactory>,
    is_default_factory: bool,
}

#[cfg(feature = "factory")]
impl FactoryProvider
{
    pub fn new(
        factory: crate::ptr::FactoryPtr<dyn crate::private::any_factory::AnyFactory>,
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
    use std::error::Error;

    use super::*;
    use crate::dependency_history::MockDependencyHistory;
    use crate::test_utils::{mocks, subjects};

    #[test]
    fn transient_type_provider_works() -> Result<(), Box<dyn Error>>
    {
        let transient_type_provider = TransientTypeProvider::<
            subjects::UserManager,
            mocks::blocking_di_container::MockDIContainer,
        >::new();

        let di_container = mocks::blocking_di_container::MockDIContainer::new();

        let dependency_history_mock = MockDependencyHistory::new();

        assert!(
            matches!(
                transient_type_provider
                    .provide(&Rc::new(di_container), dependency_history_mock)?,
                Providable::Transient(_)
            ),
            "The provided type is not transient"
        );

        Ok(())
    }

    #[test]
    fn singleton_provider_works() -> Result<(), Box<dyn Error>>
    {
        let singleton_provider =
            SingletonProvider::<
                subjects::UserManager,
                mocks::blocking_di_container::MockDIContainer,
            >::new(SingletonPtr::new(subjects::UserManager {}));

        let di_container = mocks::blocking_di_container::MockDIContainer::new();

        assert!(
            matches!(
                singleton_provider
                    .provide(&Rc::new(di_container), MockDependencyHistory::new())?,
                Providable::Singleton(_)
            ),
            "The provided type is not a singleton"
        );

        Ok(())
    }

    #[test]
    #[cfg(feature = "factory")]
    fn factory_provider_works() -> Result<(), Box<dyn Error>>
    {
        use crate::private::any_factory::AnyFactory;
        use crate::ptr::FactoryPtr;

        #[derive(Debug)]
        struct FooFactory;

        impl AnyFactory for FooFactory {}

        let factory_provider = FactoryProvider::new(FactoryPtr::new(FooFactory), false);
        let default_factory_provider =
            FactoryProvider::new(FactoryPtr::new(FooFactory), true);

        let di_container = Rc::new(mocks::blocking_di_container::MockDIContainer::new());

        assert!(
            matches!(
                factory_provider.provide(&di_container, MockDependencyHistory::new())?,
                Providable::Factory(_)
            ),
            "The provided type is not a factory"
        );

        assert!(
            matches!(
                default_factory_provider
                    .provide(&di_container, MockDependencyHistory::new())?,
                Providable::DefaultFactory(_)
            ),
            "The provided type is not a default factory"
        );

        Ok(())
    }
}
