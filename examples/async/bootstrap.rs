use anyhow::Result;
use syrette::async_di_container::AsyncDIContainer;

// Concrete implementations
use crate::animals::cat::Cat;
use crate::animals::dog::Dog;
use crate::animals::human::Human;
//
// Interfaces
use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub async fn bootstrap() -> Result<AsyncDIContainer>
{
    let mut di_container = AsyncDIContainer::new();

    di_container
        .bind::<dyn IDog>()
        .to::<Dog>()?
        .in_singleton_scope()
        .await?;

    di_container.bind::<dyn ICat>().to::<Cat>()?;
    di_container.bind::<dyn IHuman>().to::<Human>()?;

    Ok(di_container)
}
