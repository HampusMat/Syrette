use syrette::injectable;
use syrette::ptr::{FactoryPtr, TransientPtr};

use crate::interfaces::user::{IUser, IUserFactory};
use crate::interfaces::user_manager::IUserManager;

pub struct UserManager
{
    users: Vec<TransientPtr<dyn IUser>>,
    user_factory: FactoryPtr<IUserFactory>,
}

#[injectable(IUserManager)]
impl UserManager
{
    pub fn new(user_factory: FactoryPtr<IUserFactory>) -> Self
    {
        Self {
            users: Vec::new(),
            user_factory,
        }
    }
}

impl IUserManager for UserManager
{
    fn fill_with_users(&mut self)
    {
        self.users
            .push((self.user_factory)("Bob", "1983-04-13", "abc1234"));

        self.users
            .push((self.user_factory)("Anna", "1998-01-20", "IlovemYCat"));

        self.users
            .push((self.user_factory)("David", "2000-11-05", "12345678"));
    }

    fn print_users(&self)
    {
        for user in &self.users {
            println!(
                "{}, born {}, password is '{}'",
                user.get_name(),
                user.get_date_of_birth(),
                user.get_password()
            );
        }
    }
}
