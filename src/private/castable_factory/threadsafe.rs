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
    func: &'static (dyn Fn(Arc<DIContainerT>) -> TransientPtr<ReturnInterface>
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
    fn call(&self, di_container: Arc<DIContainerT>) -> TransientPtr<ReturnInterface>
    {
        (self.func)(di_container)
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

        let output = castable_factory.call(mock_di_container);

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 27 }));
    }
}
