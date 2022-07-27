use crate::interfaces::any_factory::AnyFactory;
use crate::interfaces::factory::IFactory;
use crate::ptr::TransientPtr;

pub struct CastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    func: &'static dyn Fn<Args, Output = TransientPtr<ReturnInterface>>,
}

impl<Args, ReturnInterface> CastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    pub fn new(
        func: &'static dyn Fn<Args, Output = TransientPtr<ReturnInterface>>,
    ) -> Self
    {
        Self { func }
    }
}

impl<Args, ReturnInterface> IFactory<Args, ReturnInterface>
    for CastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
}

impl<Args, ReturnInterface> Fn<Args> for CastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call(&self, args: Args) -> Self::Output
    {
        self.func.call(args)
    }
}

impl<Args, ReturnInterface> FnMut<Args> for CastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output
    {
        self.call(args)
    }
}

impl<Args, ReturnInterface> FnOnce<Args> for CastableFactory<Args, ReturnInterface>
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

impl<Args, ReturnInterface> AnyFactory for CastableFactory<Args, ReturnInterface>
where
    Args: 'static,
    ReturnInterface: 'static + ?Sized,
{
}
