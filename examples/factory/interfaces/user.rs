use syrette::factory;
use syrette::ptr::TransientPtr;

pub trait IUser
{
    fn get_name(&self) -> &'static str;
    fn get_date_of_birth(&self) -> &'static str;
    fn get_password(&self) -> &'static str;
}

#[factory]
pub type IUserFactory =
    dyn Fn(&'static str, &'static str, &'static str) -> TransientPtr<dyn IUser>;
