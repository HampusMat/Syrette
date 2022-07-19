#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use syrette::errors::di_container::DIContainerError;
use syrette::interfaces::factory::IFactory;
use syrette::ptr::{FactoryPtr, InterfacePtr};
use syrette::{factory, injectable, DIContainer};

trait IDog
{
    fn woof(&self);
}

struct Dog {}

#[injectable(IDog)]
impl Dog
{
    fn new() -> Self
    {
        Self {}
    }
}

impl IDog for Dog
{
    fn woof(&self)
    {
        println!("Woof!");
    }
}

trait ICat
{
    fn meow(&self);
}

struct Cat {}

#[injectable(ICat)]
impl Cat
{
    fn new() -> Self
    {
        Self {}
    }
}

impl ICat for Cat
{
    fn meow(&self)
    {
        println!("Meow!");
    }
}

trait ICow
{
    fn moo(&self);
}

struct Cow
{
    moo_cnt: i32,
}

impl Cow
{
    fn new(moo_cnt: i32) -> Self
    {
        Self { moo_cnt }
    }
}

impl ICow for Cow
{
    fn moo(&self)
    {
        for _ in 0..self.moo_cnt {
            println!("Moo");
        }
    }
}

#[factory]
type CowFactory = dyn IFactory<(i32,), dyn ICow>;

trait IHuman
{
    fn make_pets_make_sounds(&self);
}

struct Human
{
    dog: InterfacePtr<dyn IDog>,
    cat: InterfacePtr<dyn ICat>,
    cow_factory: FactoryPtr<CowFactory>,
}

#[injectable(IHuman)]
impl Human
{
    fn new(
        dog: InterfacePtr<dyn IDog>,
        cat: InterfacePtr<dyn ICat>,
        cow_factory: FactoryPtr<CowFactory>,
    ) -> Self
    {
        Self {
            dog,
            cat,
            cow_factory,
        }
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

        let cow: Box<dyn ICow> = (self.cow_factory)(3);

        cow.moo();
    }
}

fn main() -> error_stack::Result<(), DIContainerError>
{
    println!("Hello, world!");

    let mut di_container: DIContainer = DIContainer::new();

    di_container.bind::<dyn IDog>().to::<Dog>();
    di_container.bind::<dyn ICat>().to::<Cat>();
    di_container.bind::<dyn IHuman>().to::<Human>();

    di_container.bind::<CowFactory>().to_factory(&|moo_cnt| {
        let cow: Box<dyn ICow> = Box::new(Cow::new(moo_cnt));
        cow
    });

    let dog = di_container.get::<dyn IDog>()?;

    dog.woof();

    let human = di_container.get::<dyn IHuman>()?;

    human.make_pets_make_sounds();

    Ok(())
}
