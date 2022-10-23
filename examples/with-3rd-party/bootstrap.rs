use std::error::Error;
use std::rc::Rc;

use syrette::declare_default_factory;
use syrette::di_container::blocking::prelude::*;
use syrette::ptr::TransientPtr;
use third_party_lib::Shuriken;

use crate::interfaces::ninja::INinja;
use crate::ninja::Ninja;

declare_default_factory!(Shuriken);

pub fn bootstrap() -> Result<Rc<DIContainer>, Box<dyn Error>>
{
    let mut di_container = DIContainer::new();

    di_container.bind::<dyn INinja>().to::<Ninja>()?;

    di_container
        .bind::<Shuriken>()
        .to_default_factory(&|_| Box::new(|| TransientPtr::new(Shuriken::new())))?;

    Ok(di_container)
}
