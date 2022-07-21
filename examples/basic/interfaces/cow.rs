use syrette::factory;
use syrette::interfaces::factory::IFactory;

pub trait ICow
{
    fn moo(&self);
}

#[factory]
pub type CowFactory = dyn IFactory<(i32,), dyn ICow>;
