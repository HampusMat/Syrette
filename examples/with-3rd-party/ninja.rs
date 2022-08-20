use syrette::{injectable, ptr::TransientPtr};
use third_party_lib::IShuriken;

use crate::interfaces::ninja::INinja;

pub struct Ninja
{
    shuriken: TransientPtr<dyn IShuriken>,
}

#[injectable(INinja)]
impl Ninja
{
    pub fn new(shuriken: TransientPtr<dyn IShuriken>) -> Self
    {
        Self { shuriken }
    }
}

impl INinja for Ninja
{
    fn throw_shuriken(&self)
    {
        self.shuriken.throw();
    }
}
