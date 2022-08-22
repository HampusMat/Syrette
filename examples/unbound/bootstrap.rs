use std::error::Error;

use syrette::DIContainer;

// Concrete implementations
use crate::animal_store::AnimalStore;
use crate::animals::dog::Dog;
use crate::animals::human::Human;
//
// Interfaces
use crate::interfaces::animal_store::IAnimalStore;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub fn bootstrap() -> Result<DIContainer, Box<dyn Error>>
{
    let mut di_container: DIContainer = DIContainer::new();

    di_container
        .bind::<dyn IDog>()
        .to::<Dog>()?
        .in_singleton_scope()?;

    di_container.bind::<dyn IHuman>().to::<Human>()?;

    di_container
        .bind::<dyn IAnimalStore>()
        .to::<AnimalStore>()?;

    Ok(di_container)
}
