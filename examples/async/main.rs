#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod animals;
mod bootstrap;
mod food;
mod interfaces;

use anyhow::Result;
use syrette::di_container::asynchronous::prelude::*;
use tokio::spawn;

use crate::bootstrap::bootstrap;
use crate::interfaces::dog::IDog;
use crate::interfaces::food::IFoodFactory;
use crate::interfaces::human::IHuman;

#[tokio::main]
async fn main() -> Result<()>
{
    println!("Hello, world!");

    let di_container = bootstrap().await?;

    {
        let dog = di_container
            .get::<dyn IDog>()
            .await?
            .threadsafe_singleton()?;

        dog.woof();
    }

    let food_factory = di_container
        .get::<IFoodFactory>()
        .await?
        .threadsafe_factory()?;

    let food = food_factory();

    food.eat();

    spawn(async move {
        let human = di_container.get::<dyn IHuman>().await?.transient()?;

        human.make_pets_make_sounds();

        Ok::<_, anyhow::Error>(())
    })
    .await??;

    Ok(())
}
