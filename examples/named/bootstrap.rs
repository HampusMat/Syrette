use std::rc::Rc;

use anyhow::Result;
use syrette::DIContainer;

use crate::interfaces::ninja::INinja;
use crate::interfaces::weapon::IWeapon;
use crate::katana::Katana;
use crate::ninja::Ninja;
use crate::shuriken::Shuriken;

pub fn bootstrap() -> Result<Rc<DIContainer>>
{
    let mut di_container = DIContainer::new();

    di_container
        .bind::<dyn IWeapon>()
        .to::<Katana>()?
        .in_transient_scope()
        .when_named("strong")?;

    di_container
        .bind::<dyn IWeapon>()
        .to::<Shuriken>()?
        .in_transient_scope()
        .when_named("weak")?;

    di_container.bind::<dyn INinja>().to::<Ninja>()?;

    Ok(di_container)
}
