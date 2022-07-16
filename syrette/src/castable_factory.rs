use crate::interfaces::factory::IFactory;
use crate::libs::intertrait::CastFrom;

pub trait AnyFactory: CastFrom {}

pub struct CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
    _func: &'static dyn Fn<Args, Output = Box<Return>>,
}

impl<Args, Return> CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
    pub fn new(func: &'static dyn Fn<Args, Output = Box<Return>>) -> Self
    {
        Self { _func: func }
    }
}

impl<Args, Return> IFactory<Args, Return> for CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
}

impl<Args, Return> Fn<Args> for CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
    extern "rust-call" fn call(&self, args: Args) -> Self::Output
    {
        self._func.call(args)
    }
}

impl<Args, Return> FnMut<Args> for CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output
    {
        self.call(args)
    }
}

impl<Args, Return> FnOnce<Args> for CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
    type Output = Box<Return>;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output
    {
        self.call(args)
    }
}

impl<Args, Return> AnyFactory for CastableFactory<Args, Return>
where
    Args: 'static,
    Return: 'static + ?Sized,
{
}
