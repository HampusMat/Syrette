use std::sync::Arc;

use anyhow::Result;
use syrette::async_di_container::AsyncDIContainer;
use syrette::declare_default_factory;
use syrette::ptr::TransientPtr;

use crate::animals::cat::Cat;
use crate::animals::dog::Dog;
use crate::animals::human::Human;
use crate::food::Food;
use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;
use crate::interfaces::food::{IFood, IFoodFactory};
use crate::interfaces::human::IHuman;

declare_default_factory!(dyn ICat, threadsafe = true);

pub async fn bootstrap() -> Result<Arc<AsyncDIContainer>>
{
    let mut di_container = AsyncDIContainer::new();

    di_container
        .bind::<dyn IDog>()
        .to::<Dog>()
        .await?
        .in_singleton_scope()
        .await?;

    di_container
        .bind::<dyn ICat>()
        .to_default_factory(&|_| {
            Box::new(|| {
                let cat: TransientPtr<dyn ICat> = TransientPtr::new(Cat::new());

                cat
            })
        })
        .await?;

    di_container.bind::<dyn IHuman>().to::<Human>().await?;

    di_container
        .bind::<IFoodFactory>()
        .to_factory(&|_| {
            Box::new(|| {
                let food: Box<dyn IFood> = Box::new(Food::new());

                food
            })
        })
        .await?;

    Ok(di_container)
}
