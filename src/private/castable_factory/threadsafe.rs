use std::any::type_name;
use std::fmt::Debug;
use std::sync::Arc;

use crate::private::any_factory::{AnyFactory, AnyThreadsafeFactory};
use crate::private::factory::IThreadsafeFactory;
use crate::ptr::TransientPtr;

pub struct ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
    func: &'static (dyn Fn<(Arc<DIContainerT>,), Output = TransientPtr<ReturnInterface>>
                  + Send
                  + Sync),
}

impl<ReturnInterface, DIContainerT>
    ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
    pub fn new(
        func: &'static (dyn Fn<(Arc<DIContainerT>,), Output = TransientPtr<ReturnInterface>>
                      + Send
                      + Sync),
    ) -> Self
    {
        Self { func }
    }
}

impl<ReturnInterface, DIContainerT> IThreadsafeFactory<ReturnInterface, DIContainerT>
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

impl<ReturnInterface, DIContainerT> Fn<(Arc<DIContainerT>,)>
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call(&self, args: (Arc<DIContainerT>,)) -> Self::Output
    {
        self.func.call(args)
    }
}

impl<ReturnInterface, DIContainerT> FnMut<(Arc<DIContainerT>,)>
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call_mut(&mut self, args: (Arc<DIContainerT>,))
        -> Self::Output
    {
        self.call(args)
    }
}

impl<ReturnInterface, DIContainerT> FnOnce<(Arc<DIContainerT>,)>
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
    type Output = TransientPtr<ReturnInterface>;

    extern "rust-call" fn call_once(self, args: (Arc<DIContainerT>,)) -> Self::Output
    {
        self.call(args)
    }
}

impl<ReturnInterface, DIContainerT> AnyFactory
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

impl<ReturnInterface, DIContainerT> AnyThreadsafeFactory
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

impl<ReturnInterface, DIContainerT> Debug
    for ThreadsafeCastableFactory<ReturnInterface, DIContainerT>
where
    DIContainerT: 'static,
    ReturnInterface: 'static + ?Sized,
{
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let ret = type_name::<TransientPtr<ReturnInterface>>();

        formatter.write_fmt(format_args!(
            "ThreadsafeCastableFactory (Arc<AsyncDIContainer>) -> {ret} {{ ... }}",
        ))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::di_container::asynchronous::MockAsyncDIContainer;

    #[derive(Debug, PartialEq, Eq)]
    struct Bacon
    {
        heal_amount: u32,
    }

    #[test]
    fn can_call()
    {
        let castable_factory = ThreadsafeCastableFactory::new(&|_| {
            TransientPtr::new(Bacon { heal_amount: 27 })
        });

        let mock_di_container = Arc::new(MockAsyncDIContainer::new());

        let output = castable_factory.call((mock_di_container,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 27 }));
    }

    #[test]
    fn can_call_mut()
    {
        let mut castable_factory = ThreadsafeCastableFactory::new(&|_| {
            TransientPtr::new(Bacon { heal_amount: 1092 })
        });

        let mock_di_container = Arc::new(MockAsyncDIContainer::new());

        let output = castable_factory.call_mut((mock_di_container,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 1092 }));
    }

    #[test]
    fn can_call_once()
    {
        let castable_factory = ThreadsafeCastableFactory::new(&|_| {
            TransientPtr::new(Bacon { heal_amount: 547 })
        });

        let mock_di_container = Arc::new(MockAsyncDIContainer::new());

        let output = castable_factory.call_once((mock_di_container,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 547 }));
    }
}
