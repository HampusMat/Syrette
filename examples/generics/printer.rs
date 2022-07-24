use std::fmt::Display;

use syrette::injectable;

use crate::interfaces::printer::IPrinter;

pub struct Printer {}

#[injectable]
impl Printer
{
    pub fn new() -> Self
    {
        Self {}
    }
}

impl<Printable: Display> IPrinter<Printable> for Printer
{
    fn print(&self, out: Printable)
    {
        println!("{}", out);
    }
}
