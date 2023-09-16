use std::any::type_name;
use std::fmt::Debug;
use std::rc::Rc;

use crate::private::any_factory::AnyFactory;
use crate::private::factory::IFactory;
use crate::ptr::TransientPtr;

pub struct CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
    DIContainerT: 'static,
{
    func: &'static dyn Fn<(Rc<DIContainerT>,), Output = TransientPtr<ReturnInterface>>,
}

impl<ReturnInterface, DIContainerT> CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    pub fn new(
        func: &'static dyn Fn<
            (Rc<DIContainerT>,),
            Output = TransientPtr<ReturnInterface>,
        >,
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
}

impl<ReturnInterface, DIContainerT> Fn<(Rc<DIContainerT>,)>
    for CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call(&self, args: (Rc<DIContainerT>,)) -> Self::Output
    {
        self.func.call(args)
    }
}

impl<ReturnInterface, DIContainerT> FnMut<(Rc<DIContainerT>,)>
    for CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call_mut(&mut self, args: (Rc<DIContainerT>,)) -> Self::Output
    {
        self.call(args)
    }
}

impl<ReturnInterface, DIContainerT> FnOnce<(Rc<DIContainerT>,)>
    for CastableFactory<ReturnInterface, DIContainerT>
where
    ReturnInterface: 'static + ?Sized,
{
    type Output = TransientPtr<ReturnInterface>;

    extern "rust-call" fn call_once(self, args: (Rc<DIContainerT>,)) -> Self::Output
    {
        self.call(args)
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

        formatter.write_fmt(format_args!("CastableFactory (Rc<DIContainer>) -> {ret}"))
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
        let castable_factory =
            CastableFactory::new(&|_| TransientPtr::new(Bacon { heal_amount: 27 }));

        let mock_di_container = Rc::new(MockDIContainer::new());

        let output = castable_factory.call((mock_di_container,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 27 }));
    }

    #[test]
    fn can_call_mut()
    {
        let mut castable_factory =
            CastableFactory::new(&|_| TransientPtr::new(Bacon { heal_amount: 103 }));

        let mock_di_container = Rc::new(MockDIContainer::new());

        let output = castable_factory.call_mut((mock_di_container,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 103 }));
    }

    #[test]
    fn can_call_once()
    {
        let castable_factory =
            CastableFactory::new(&|_| TransientPtr::new(Bacon { heal_amount: 19 }));

        let mock_di_container = Rc::new(MockDIContainer::new());

        let output = castable_factory.call_once((mock_di_container,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 19 }));
    }
}
