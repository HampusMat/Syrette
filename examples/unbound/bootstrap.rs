use syrette::DIContainer;

// Concrete implementations
use crate::animals::dog::Dog;
use crate::animals::human::Human;
//
// Interfaces
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub fn bootstrap() -> DIContainer
{
    let mut di_container: DIContainer = DIContainer::new();

    di_container
        .bind::<dyn IDog>()
        .to_singleton::<Dog>()
        .unwrap();

    di_container.bind::<dyn IHuman>().to::<Human>().unwrap();

    di_container
}
