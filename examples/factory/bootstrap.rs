use std::error::Error;
use std::rc::Rc;

use syrette::ptr::TransientPtr;
use syrette::DIContainer;

use crate::interfaces::user::{IUser, IUserFactory};
use crate::interfaces::user_manager::IUserManager;
use crate::user::User;
use crate::user_manager::UserManager;

pub fn bootstrap() -> Result<Rc<DIContainer>, Box<dyn Error>>
{
    let mut di_container = DIContainer::new();

    di_container
        .bind::<dyn IUserManager>()
        .to::<UserManager>()?;

    di_container.bind::<IUserFactory>().to_factory(&|_| {
        Box::new(move |name, date_of_birth, password| {
            let user: TransientPtr<dyn IUser> =
                TransientPtr::new(User::new(name, date_of_birth, password));

            user
        })
    })?;

    Ok(di_container)
}
