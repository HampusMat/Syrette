use std::any::{type_name, Any};
use std::fmt::Debug;

use crate::ptr::TransientPtr;

#[cfg(feature = "async")]
pub mod threadsafe;

/// Interface for any castable function.
pub trait AnyCastableFunction: Any + Debug
{
    fn as_any(&self) -> &dyn Any;
}

pub struct CastableFunction<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
    DIContainerT: 'static,
{
    func: &'static dyn Fn(&DIContainerT) -> TransientPtr<ReturnInterface>,
}

impl<ReturnInterface, DIContainerT> CastableFunction<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    pub fn new(
        func: &'static dyn Fn(&DIContainerT) -> TransientPtr<ReturnInterface>,
    ) -> Self
    {
        Self { func }
    }

    pub fn call(&self, di_container: &DIContainerT) -> TransientPtr<ReturnInterface>
    {
        (self.func)(di_container)
    }
}

impl<ReturnInterface, DIContainerT> AnyCastableFunction
    for CastableFunction<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
    DIContainerT: 'static,
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }
}

impl<ReturnInterface, DIContainerT> Debug
    for CastableFunction<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let ret = type_name::<TransientPtr<ReturnInterface>>();

        formatter.write_fmt(format_args!(
            "CastableFunction (&DIContainer) -> {ret} {{ ... }}"
        ))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::di_container::blocking::MockDIContainer;

    #[derive(Debug, PartialEq, Eq)]
    struct Bacon
    {
        heal_amount: u32,
    }

    #[test]
    fn can_call()
    {
        let castable_func = CastableFunction::new(&|_: &MockDIContainer| {
            TransientPtr::new(Bacon { heal_amount: 27 })
        });

        let mock_di_container = MockDIContainer::new();

        let output = castable_func.call(&mock_di_container);

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 27 }));
    }
}
