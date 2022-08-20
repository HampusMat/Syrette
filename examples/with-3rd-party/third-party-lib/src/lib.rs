pub trait IShuriken
{
    fn throw(&self);
}

pub struct Shuriken {}

impl Shuriken
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    {
        Self {}
    }
}

impl IShuriken for Shuriken
{
    fn throw(&self)
    {
        println!("Threw shuriken!");
    }
}
