mod bootstrap;
mod interfaces;
mod printer;

use bootstrap::bootstrap;
use interfaces::printer::IPrinter;

fn main()
{
    let di_container = bootstrap();

    let string_printer = di_container.get::<dyn IPrinter<String>>().unwrap();

    string_printer.print("Hello there".to_string());

    let int_printer = di_container.get::<dyn IPrinter<i32>>().unwrap();

    int_printer.print(2782028);
}
