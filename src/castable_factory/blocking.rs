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

#[cfg(test)]
mod tests
{
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Bacon
    {
        heal_amount: u32,
    }

    #[test]
    fn can_call()
    {
        let castable_factory =
            CastableFactory::new(&|heal_amount| TransientPtr::new(Bacon { heal_amount }));

        let output = castable_factory.call((27,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 27 }));
    }

    #[test]
    fn can_call_mut()
    {
        let mut castable_factory =
            CastableFactory::new(&|heal_amount| TransientPtr::new(Bacon { heal_amount }));

        let output = castable_factory.call_mut((103,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 103 }));
    }

    #[test]
    fn can_call_once()
    {
        let castable_factory =
            CastableFactory::new(&|heal_amount| TransientPtr::new(Bacon { heal_amount }));

        let output = castable_factory.call_once((19,));

        assert_eq!(output, TransientPtr::new(Bacon { heal_amount: 19 }));
    }
}
