use crate::interfaces::user::IUser;

pub struct User
{
    name: &'static str,
    date_of_birth: &'static str,
    password: &'static str,
}

impl User
{
    pub fn new(
        name: &'static str,
        date_of_birth: &'static str,
        password: &'static str,
    ) -> Self
    {
        Self {
            name,
            date_of_birth,
            password,
        }
    }
}

impl IUser for User
{
    fn get_name(&self) -> &'static str
    {
        self.name
    }

    fn get_date_of_birth(&self) -> &'static str
    {
        self.date_of_birth
    }

    fn get_password(&self) -> &'static str
    {
        self.password
    }
}
