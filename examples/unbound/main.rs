#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod animal_store;
mod animals;
mod bootstrap;
mod interfaces;

use anyhow::Result;
use syrette::di_container::blocking::prelude::*;

use crate::bootstrap::bootstrap;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

fn main() -> Result<()>
{
    println!("Hello, world!");

    let di_container = bootstrap()?;

    let dog = di_container.get::<dyn IDog>()?.singleton()?;

    dog.woof();

    let human = di_container.get::<dyn IHuman>()?.transient()?;

    human.make_pets_make_sounds();

    Ok(())
}
