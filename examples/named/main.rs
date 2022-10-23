#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod bootstrap;
mod interfaces;
mod katana;
mod ninja;
mod shuriken;

use anyhow::Result;
use syrette::di_container::blocking::prelude::*;

use crate::bootstrap::bootstrap;
use crate::interfaces::ninja::INinja;

fn main() -> Result<()>
{
    let di_container = bootstrap()?;

    let ninja = di_container.get::<dyn INinja>()?.transient()?;

    ninja.use_weapons();

    Ok(())
}
