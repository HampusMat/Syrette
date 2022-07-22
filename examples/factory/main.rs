#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod bootstrap;
mod interfaces;
mod user;

use bootstrap::bootstrap;
use interfaces::user::IUser;
use interfaces::user::IUserFactory;
use syrette::ptr::FactoryPtr;
use syrette::ptr::InterfacePtr;

fn add_users(
    users: &mut Vec<InterfacePtr<dyn IUser>>,
    user_factory: &FactoryPtr<IUserFactory>,
)
{
    users.push(user_factory("Bob", "1983-04-13", "abc1234"));
    users.push(user_factory("Anna", "1998-01-20", "IlovemYCat"));
    users.push(user_factory("David", "2000-11-05", "12345678"));
}

fn main()
{
    println!("Hello, world!");

    let di_container = bootstrap();

    let user_factory = di_container.get_factory::<IUserFactory>().unwrap();

    let mut users = Vec::<InterfacePtr<dyn IUser>>::new();

    add_users(&mut users, &user_factory);

    println!("Printing user information");

    for user in users {
        println!(
            "{}, born {}, password is '{}'",
            user.get_name(),
            user.get_date_of_birth(),
            user.get_password()
        );
    }
}
