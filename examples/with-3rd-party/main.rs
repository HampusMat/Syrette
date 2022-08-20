#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::fmt::Display;

mod bootstrap;
mod interfaces;
mod ninja;

use error_stack::{Context, ResultExt};

use crate::bootstrap::bootstrap;
use crate::interfaces::ninja::INinja;

#[derive(Debug)]
struct ApplicationError;

impl Display for ApplicationError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("An application error has occurred")
    }
}

impl Context for ApplicationError {}

fn main() -> error_stack::Result<(), ApplicationError>
{
    println!("Hello, world!");

    let di_container = bootstrap().change_context(ApplicationError)?;

    let ninja = di_container
        .get::<dyn INinja>()
        .change_context(ApplicationError)?;

    ninja.throw_shuriken();

    Ok(())
}
