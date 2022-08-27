use syrette::injectable;

use crate::interfaces::weapon::IWeapon;

pub struct Shuriken {}

#[injectable(IWeapon)]
impl Shuriken
{
    pub fn new() -> Self
    {
        Self {}
    }
}

impl IWeapon for Shuriken
{
    fn use_it(&self)
    {
        println!("Used shuriken!");
    }
}
