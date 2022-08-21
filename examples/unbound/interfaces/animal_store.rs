use syrette::ptr::{SingletonPtr, TransientPtr};

use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;

pub trait IAnimalStore
{
    fn get_dog(&self) -> SingletonPtr<dyn IDog>;

    fn get_cat(&self) -> &TransientPtr<dyn ICat>;
}
