use syrette::errors::di_container::BindingBuilderError;
use syrette::ptr::TransientPtr;
use syrette::{declare_default_factory, DIContainer};
use third_party_lib::Shuriken;

// Interfaces
use crate::interfaces::ninja::INinja;
//
// Concrete implementations
use crate::ninja::Ninja;

declare_default_factory!(Shuriken);

pub fn bootstrap() -> error_stack::Result<DIContainer, BindingBuilderError>
{
    let mut di_container: DIContainer = DIContainer::new();

    di_container.bind::<dyn INinja>().to::<Ninja>()?;

    di_container
        .bind::<Shuriken>()
        .to_default_factory(&|| TransientPtr::new(Shuriken::new()))?;

    Ok(di_container)
}
