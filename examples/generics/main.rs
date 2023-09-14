mod bootstrap;
mod interfaces;
mod printer;

use std::error::Error;

use crate::bootstrap::bootstrap;
use crate::interfaces::printer::IPrinter;

fn main() -> Result<(), Box<dyn Error>>
{
    let di_container = bootstrap();

    let string_printer = di_container.get::<dyn IPrinter<String>>()?.transient()?;

    string_printer.print("Hello there".to_string());

    let int_printer = di_container.get::<dyn IPrinter<i32>>()?.transient()?;

    int_printer.print(2782028);

    Ok(())
}
