use crate::interfaces::cow::ICow;

pub struct Cow
{
    moo_cnt: i32,
}

impl Cow
{
    pub fn new(moo_cnt: i32) -> Self
    {
        Self { moo_cnt }
    }
}

impl ICow for Cow
{
    fn moo(&self)
    {
        for _ in 0..self.moo_cnt {
            println!("Moo");
        }
    }
}
