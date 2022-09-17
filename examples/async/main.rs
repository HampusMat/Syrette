#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use anyhow::Result;
use tokio::spawn;

mod animals;
mod bootstrap;
mod interfaces;

use bootstrap::bootstrap;
use interfaces::dog::IDog;
use interfaces::human::IHuman;

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

    spawn(async move {
        let human = di_container.get::<dyn IHuman>().await?.transient()?;

        human.make_pets_make_sounds();

        Ok::<_, anyhow::Error>(())
    })
    .await??;

    Ok(())
}
