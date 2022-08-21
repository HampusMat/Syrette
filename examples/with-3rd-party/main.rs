#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::error::Error;

mod bootstrap;
mod interfaces;
mod ninja;

use crate::bootstrap::bootstrap;
use crate::interfaces::ninja::INinja;

fn main() -> Result<(), Box<dyn Error>>
{
    println!("Hello, world!");

    let di_container = bootstrap()?;

    let ninja = di_container.get::<dyn INinja>()?;

    ninja.throw_shuriken();

    Ok(())
}
