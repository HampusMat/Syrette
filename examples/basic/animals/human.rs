use syrette::injectable;
use syrette::ptr::TransientPtr;

use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub struct Human
{
    dog: TransientPtr<dyn IDog>,
    cat: TransientPtr<dyn ICat>,
}

#[injectable(IHuman)]
impl Human
{
    pub fn new(dog: TransientPtr<dyn IDog>, cat: TransientPtr<dyn ICat>) -> Self
    {
        Self { dog, cat }
    }
}

impl IHuman for Human
{
    fn make_pets_make_sounds(&self)
    {
        println!("Hi doggy!");

        self.dog.woof();

        println!("Hi kitty!");

        self.cat.meow();
    }
}
