use syrette::injectable;

use crate::interfaces::dog::IDog;

pub struct Dog {}

#[injectable(IDog)]
impl Dog
{
    pub fn new() -> Self
    {
        Self {}
    }
}

impl IDog for Dog
{
    fn woof(&self)
    {
        println!("Woof!");
    }
}
