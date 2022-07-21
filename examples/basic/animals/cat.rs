use syrette::injectable;

use crate::interfaces::cat::ICat;

pub struct Cat {}

#[injectable(ICat)]
impl Cat
{
    pub fn new() -> Self
    {
        Self {}
    }
}

impl ICat for Cat
{
    fn meow(&self)
    {
        println!("Meow!");
    }
}
