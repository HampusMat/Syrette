#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod bootstrap;
mod interfaces;
mod user;
mod user_manager;

use std::error::Error;

use bootstrap::bootstrap;

use crate::interfaces::user_manager::IUserManager;

fn main() -> Result<(), Box<dyn Error>>
{
    println!("Hello, world!");

    let di_container = bootstrap()?;

    let mut user_manager = di_container.get::<dyn IUserManager>()?.transient()?;

    user_manager.fill_with_users();

    println!("Printing user information");

    user_manager.print_users();

    Ok(())
}
