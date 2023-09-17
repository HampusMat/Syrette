use std::error::Error;

use syrette::ptr::TransientPtr;
use syrette::{declare_default_factory, DIContainer};
use third_party_lib::Shuriken;

use crate::interfaces::ninja::INinja;
use crate::ninja::Ninja;

declare_default_factory!(Shuriken);

pub fn bootstrap() -> Result<DIContainer, Box<dyn Error>>
{
    let mut di_container = DIContainer::new();

    di_container.bind::<dyn INinja>().to::<Ninja>()?;

    di_container
        .bind::<Shuriken>()
        .to_default_factory(&|_| Box::new(|| TransientPtr::new(Shuriken::new())))?;

    Ok(di_container)
}
