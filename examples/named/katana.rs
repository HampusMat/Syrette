use syrette::injectable;

use crate::interfaces::weapon::IWeapon;

pub struct Katana {}

#[injectable(IWeapon)]
impl Katana
{
    pub fn new() -> Self
    {
        Self {}
    }
}

impl IWeapon for Katana
{
    fn use_it(&self)
    {
        println!("Used katana!");
    }
}
