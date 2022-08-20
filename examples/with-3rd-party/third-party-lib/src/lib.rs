pub struct Shuriken {}

impl Shuriken
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    {
        Self {}
    }

    pub fn throw(&self)
    {
        println!("Threw shuriken!");
    }
}
