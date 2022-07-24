use std::fmt::Display;

pub trait IPrinter<Printable: Display>
{
    fn print(&self, out: Printable);
}
