use std::rc::Rc;

use anyhow::Result;
use syrette::di_container::blocking::prelude::*;

use crate::animal_store::AnimalStore;
use crate::animals::dog::Dog;
use crate::animals::human::Human;
use crate::interfaces::animal_store::IAnimalStore;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub fn bootstrap() -> Result<Rc<DIContainer>>
{
    let mut di_container = DIContainer::new();

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
