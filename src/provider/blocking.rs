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
    Function(
        std::rc::Rc<dyn crate::castable_function::AnyCastableFunction>,
        ProvidableFunctionKind,
    ),
}

#[cfg(feature = "factory")]
#[derive(Debug, Clone, Copy)]
pub enum ProvidableFunctionKind
{
    Instant,
    UserCalled,
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
pub struct FunctionProvider
{
    function: std::rc::Rc<dyn crate::castable_function::AnyCastableFunction>,
    providable_func_kind: ProvidableFunctionKind,
}

#[cfg(feature = "factory")]
impl FunctionProvider
{
    pub fn new(
        function: std::rc::Rc<dyn crate::castable_function::AnyCastableFunction>,
        providable_func_kind: ProvidableFunctionKind,
    ) -> Self
    {
        Self {
            function,
            providable_func_kind,
        }
    }
}

#[cfg(feature = "factory")]
impl<DIContainerType> IProvider<DIContainerType> for FunctionProvider
{
    fn provide(
        &self,
        _di_container: &DIContainerType,
        _dependency_history: DependencyHistory,
    ) -> Result<Providable<DIContainerType>, InjectableError>
    {
        Ok(Providable::Function(
            self.function.clone(),
            self.providable_func_kind,
        ))
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
    fn function_provider_works()
    {
        use std::any::Any;
        use std::rc::Rc;

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

        let user_called_func_provider = FunctionProvider::new(
            Rc::new(FooFactory),
            ProvidableFunctionKind::UserCalled,
        );

        let instant_func_provider =
            FunctionProvider::new(Rc::new(FooFactory), ProvidableFunctionKind::Instant);

        let di_container = MockDIContainer::new();

        assert!(
            matches!(
                user_called_func_provider
                    .provide(&di_container, MockDependencyHistory::new()),
                Ok(Providable::Function(_, ProvidableFunctionKind::UserCalled))
            ),
            concat!(
                "The provided type is not a Providable::Function of kind ",
                "ProvidableFunctionKind::UserCalled"
            )
        );

        assert!(
            matches!(
                instant_func_provider
                    .provide(&di_container, MockDependencyHistory::new()),
                Ok(Providable::Function(_, ProvidableFunctionKind::Instant))
            ),
            concat!(
                "The provided type is not a Providable::Function of kind ",
                "ProvidableFunctionKind::Instant"
            )
        );
    }
}
