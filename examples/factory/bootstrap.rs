use syrette::ptr::TransientPtr;
use syrette::DIContainer;

// Interfaces
use crate::interfaces::user::{IUser, IUserFactory};
//
// Concrete implementations
use crate::user::User;

pub fn bootstrap() -> DIContainer
{
    let mut di_container: DIContainer = DIContainer::new();

    di_container
        .bind::<IUserFactory>()
        .to_factory(&|name, date_of_birth, password| {
            let user: TransientPtr<dyn IUser> =
                TransientPtr::new(User::new(name, date_of_birth, password));

            user
        });

    di_container
}
