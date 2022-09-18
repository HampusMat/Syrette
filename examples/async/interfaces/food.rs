use syrette::factory;

pub trait IFood
{
    fn eat(&self);
}

#[factory(threadsafe = true)]
pub type IFoodFactory = dyn Fn() -> dyn IFood;
