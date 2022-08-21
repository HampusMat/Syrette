use syrette::injectable;
use syrette::ptr::{SingletonPtr, TransientPtr};

use crate::interfaces::animal_store::IAnimalStore;
use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;

pub struct AnimalStore
{
    dog: SingletonPtr<dyn IDog>,
    cat: TransientPtr<dyn ICat>,
}

#[injectable]
impl AnimalStore
{
    fn new(dog: SingletonPtr<dyn IDog>, cat: TransientPtr<dyn ICat>) -> Self
    {
        Self { dog, cat }
    }
}

impl IAnimalStore for AnimalStore
{
    fn get_dog(&self) -> SingletonPtr<dyn IDog>
    {
        self.dog.clone()
    }

    fn get_cat(&self) -> &TransientPtr<dyn ICat>
    {
        &self.cat
    }
}
