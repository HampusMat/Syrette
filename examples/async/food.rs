use crate::interfaces::food::IFood;

pub struct Food {}

impl Food
{
    pub fn new() -> Self
    {
        Self {}
    }
}

impl IFood for Food
{
    fn eat(&self)
    {
        println!("Ate food");
    }
}
