use syrette::factory;
use syrette::ptr::TransientPtr;

pub trait IFood: Send + Sync
{
    fn eat(&self);
}

#[factory(threadsafe = true)]
pub type IFoodFactory = dyn Fn() -> TransientPtr<dyn IFood> + Send + Sync;
