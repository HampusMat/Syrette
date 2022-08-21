#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::error::Error;

mod animal_store;
mod animals;
mod bootstrap;
mod interfaces;

use bootstrap::bootstrap;
use interfaces::dog::IDog;
use interfaces::human::IHuman;

fn main() -> Result<(), Box<dyn Error>>
{
    println!("Hello, world!");

    let di_container = bootstrap();

    let dog = di_container.get_singleton::<dyn IDog>().unwrap();

    dog.woof();

    let human = di_container.get::<dyn IHuman>()?;

    human.make_pets_make_sounds();

    Ok(())
}
