use std::any::type_name;
use std::fmt::Debug;

use crate::private::any_factory::AnyFactory;
use crate::private::factory::IFactory;
use crate::ptr::TransientPtr;

pub struct CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
    DIContainerT: 'static,
{
    func: &'static dyn Fn(&DIContainerT) -> TransientPtr<ReturnInterface>,
}

impl<ReturnInterface, DIContainerT> CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    pub fn new(
        func: &'static dyn Fn(&DIContainerT) -> TransientPtr<ReturnInterface>,
    ) -> Self
    {
        Self { func }
    }
}

impl<ReturnInterface, DIContainerT> IFactory<ReturnInterface, DIContainerT>
    for CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    fn call(&self, di_container: &DIContainerT) -> TransientPtr<ReturnInterface>
    {
        (self.func)(di_container)
    }
}

impl<ReturnInterface, DIContainerT> AnyFactory
    for CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
    DIContainerT: 'static,
{
}

impl<ReturnInterface, DIContainerT> Debug
    for CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let ret = type_name::<TransientPtr<ReturnInterface>>();

        formatter.write_fmt(format_args!(
            "CastableFactory (&DIContainer) -> {ret} {{ ... }}"
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
        let castable_factory = CastableFactory::new(&|_: &MockDIContainer| {
            TransientPtr::new(Bacon { heal_amount: 27 })
        });

        let mock_di_container = MockDIContainer::new();

        let output = castable_factory.call(&mock_di_container);

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 27 }));
    }
}
