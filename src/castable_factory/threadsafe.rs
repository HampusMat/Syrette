use crate::interfaces::any_factory::{AnyFactory, AnyThreadsafeFactory};
use crate::interfaces::factory::IThreadsafeFactory;
use crate::ptr::TransientPtr;

pub struct ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    func: &'static (dyn Fn<Args, Output = TransientPtr<ReturnInterface>> + Send + Sync),
}

impl<Args, ReturnInterface> ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    pub fn new(
        func: &'static (dyn Fn<Args, Output = TransientPtr<ReturnInterface>>
                      + Send
                      + Sync),
    ) -> Self
    {
        Self { func }
    }
}

impl<Args, ReturnInterface> IThreadsafeFactory<Args, ReturnInterface>
    for ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

impl<Args, ReturnInterface> Fn<Args> for ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call(&self, args: Args) -> Self::Output
    {
        self.func.call(args)
    }
}

impl<Args, ReturnInterface> FnMut<Args>
    for ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output
    {
        self.call(args)
    }
}

impl<Args, ReturnInterface> FnOnce<Args>
    for ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    type Output = TransientPtr<ReturnInterface>;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output
    {
        self.call(args)
    }
}

impl<Args, ReturnInterface> AnyFactory
    for ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

impl<Args, ReturnInterface> AnyThreadsafeFactory
    for ThreadsafeCastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn can_call()
    {
        #[derive(Debug, PartialEq, Eq)]
        struct Bacon
        {
            heal_amount: u32,
        }

        let castable_factory = ThreadsafeCastableFactory::new(&|heal_amount| {
            TransientPtr::new(Bacon { heal_amount })
        });

        let output = castable_factory(27);

        assert_eq!(output, Box::new(Bacon { heal_amount: 27 }));
    }
}
