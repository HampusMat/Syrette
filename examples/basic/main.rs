#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use syrette::errors::di_container::DIContainerError;

mod animals;
mod bootstrap;
mod interfaces;

use bootstrap::bootstrap;
use interfaces::dog::IDog;
use interfaces::human::IHuman;

fn main() -> error_stack::Result<(), DIContainerError>
{
    println!("Hello, world!");

    let di_container = bootstrap();

    let dog = di_container.get::<dyn IDog>()?;

    dog.woof();

    let human = di_container.get::<dyn IHuman>()?;

    human.make_pets_make_sounds();

    Ok(())
}
