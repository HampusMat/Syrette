use syrette::ptr::TransientPtr;

pub trait IFood: Send + Sync
{
    fn eat(&self);
}

pub type IFoodFactory = dyn Fn() -> TransientPtr<dyn IFood> + Send + Sync;
