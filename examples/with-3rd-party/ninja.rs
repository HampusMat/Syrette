use syrette::{injectable, ptr::TransientPtr};
use third_party_lib::Shuriken;

use crate::interfaces::ninja::INinja;

pub struct Ninja
{
    shuriken: TransientPtr<Shuriken>,
}

#[injectable(INinja)]
impl Ninja
{
    pub fn new(shuriken: TransientPtr<Shuriken>) -> Self
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
