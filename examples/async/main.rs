#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::sync::Arc;

use anyhow::Result;
use tokio::spawn;
use tokio::sync::Mutex;

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

    let di_container = Arc::new(Mutex::new(bootstrap().await?));

    {
        let dog = di_container
            .lock()
            .await
            .get::<dyn IDog>()
            .await?
            .threadsafe_singleton()?;

        dog.woof();
    }

    spawn(async move {
        let human = di_container
            .lock()
            .await
            .get::<dyn IHuman>()
            .await?
            .transient()?;

        human.make_pets_make_sounds();

        Ok::<_, anyhow::Error>(())
    })
    .await??;

    Ok(())
}
