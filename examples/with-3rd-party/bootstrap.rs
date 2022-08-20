use syrette::errors::di_container::BindingBuilderError;
use syrette::ptr::TransientPtr;
use syrette::{declare_default_factory, DIContainer};
use third_party_lib::{IShuriken, Shuriken};

// Interfaces
use crate::interfaces::ninja::INinja;
//
// Concrete implementations
use crate::ninja::Ninja;

declare_default_factory!(IShuriken);

pub fn bootstrap() -> error_stack::Result<DIContainer, BindingBuilderError>
{
    let mut di_container: DIContainer = DIContainer::new();

    di_container.bind::<dyn INinja>().to::<Ninja>()?;

    di_container
        .bind::<dyn IShuriken>()
        .to_default_factory(&|| {
            let shuriken: TransientPtr<dyn IShuriken> =
                TransientPtr::new(Shuriken::new());

            shuriken
        })?;

    Ok(di_container)
}
