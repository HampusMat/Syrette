use syrette::factory;
use syrette::interfaces::factory::IFactory;

pub trait IUser
{
    fn get_name(&self) -> &'static str;
    fn get_date_of_birth(&self) -> &'static str;
    fn get_password(&self) -> &'static str;
}

#[factory]
pub type IUserFactory =
    dyn IFactory<(&'static str, &'static str, &'static str), dyn IUser>;
