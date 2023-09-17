use std::error::Error;

use syrette::DIContainer;

use crate::animals::cat::Cat;
use crate::animals::dog::Dog;
use crate::animals::human::Human;
use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub fn bootstrap() -> Result<DIContainer, Box<dyn Error>>
{
    let mut di_container = DIContainer::new();

    di_container
        .bind::<dyn IDog>()
        .to::<Dog>()?
        .in_singleton_scope()?;

    di_container.bind::<dyn ICat>().to::<Cat>()?;
    di_container.bind::<dyn IHuman>().to::<Human>()?;

    Ok(di_container)
}
