use syrette::injectable;
use syrette::ptr::TransientPtr;

use crate::interfaces::animal_store::IAnimalStore;
use crate::interfaces::human::IHuman;

pub struct Human
{
    animal_store: TransientPtr<dyn IAnimalStore>,
}

#[injectable(IHuman)]
impl Human
{
    pub fn new(animal_store: TransientPtr<dyn IAnimalStore>) -> Self
    {
        Self { animal_store }
    }
}

impl IHuman for Human
{
    fn make_pets_make_sounds(&self)
    {
        let dog = self.animal_store.get_dog();

        println!("Hi doggy!");

        dog.woof();

        let cat = self.animal_store.get_cat();

        println!("Hi kitty!");

        cat.meow();
    }
}
