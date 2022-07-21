use syrette::injectable;
use syrette::ptr::{FactoryPtr, InterfacePtr};

use crate::interfaces::cat::ICat;
use crate::interfaces::cow::{CowFactory, ICow};
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub struct Human
{
    dog: InterfacePtr<dyn IDog>,
    cat: InterfacePtr<dyn ICat>,
    cow_factory: FactoryPtr<CowFactory>,
}

#[injectable(IHuman)]
impl Human
{
    pub fn new(
        dog: InterfacePtr<dyn IDog>,
        cat: InterfacePtr<dyn ICat>,
        cow_factory: FactoryPtr<CowFactory>,
    ) -> Self
    {
        Self {
            dog,
            cat,
            cow_factory,
        }
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

        let cow: Box<dyn ICow> = (self.cow_factory)(3);

        cow.moo();
    }
}
