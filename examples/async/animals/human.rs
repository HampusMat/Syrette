use std::time::Duration;

use syrette::injectable;
use syrette::ptr::{ThreadsafeSingletonPtr, TransientPtr};
use tokio::time::sleep;

use crate::interfaces::cat::ICat;
use crate::interfaces::dog::IDog;
use crate::interfaces::human::IHuman;

pub struct Human
{
    dog: ThreadsafeSingletonPtr<dyn IDog>,
    cat: TransientPtr<dyn ICat>,
}

#[injectable(IHuman, async = true)]
impl Human
{
    pub async fn new(
        dog: ThreadsafeSingletonPtr<dyn IDog>,
        cat: TransientPtr<dyn ICat>,
    ) -> Self
    {
        // The human needs some rest first
        sleep(Duration::from_secs(1)).await;

        Self { dog, cat }
    }
}

impl IHuman for Human
{
    fn make_pets_make_sounds(&self)
    {
        println!("Hi doggy!");

        self.dog.woof();

        println!("Hi kitty!");

        self.cat.meow();
    }
}
