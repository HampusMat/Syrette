use syrette::injectable;
use syrette::ptr::TransientPtr;

use crate::interfaces::ninja::INinja;
use crate::interfaces::weapon::IWeapon;

pub struct Ninja
{
    strong_weapon: TransientPtr<dyn IWeapon>,
    weak_weapon: TransientPtr<dyn IWeapon>,
}

#[injectable(INinja)]
impl Ninja
{
    pub fn new(
        #[rustfmt::skip] // Prevent rustfmt from turning this into a single line
        #[syrette::named("strong")]
        strong_weapon: TransientPtr<dyn IWeapon>,

        #[rustfmt::skip] // Prevent rustfmt from turning this into a single line
        #[named("weak")]
        weak_weapon: TransientPtr<dyn IWeapon>,
    ) -> Self
    {
        Self {
            strong_weapon,
            weak_weapon,
        }
    }
}

impl INinja for Ninja
{
    fn use_weapons(&self)
    {
        println!("Ninja is using his strong weapon!");

        self.strong_weapon.use_it();

        println!("Ninja is using his weak weapon!");

        self.weak_weapon.use_it();
    }
}
