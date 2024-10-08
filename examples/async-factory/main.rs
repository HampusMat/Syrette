#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::time::Duration;

use anyhow::Result;
use syrette::future::BoxFuture;
use syrette::ptr::TransientPtr;
use syrette::AsyncDIContainer;
use tokio::time::sleep;

trait IFoo: Send + Sync
{
    fn bar(&self);
}

type IFooFactory =
    dyn Fn(i32) -> BoxFuture<'static, TransientPtr<dyn IFoo>> + Send + Sync;

struct Foo
{
    cnt: i32,
}

impl Foo
{
    fn new(cnt: i32) -> Self
    {
        Self { cnt }
    }
}

impl IFoo for Foo
{
    fn bar(&self)
    {
        for _ in 1..self.cnt {
            println!("Foobar");
        }
    }
}

trait IPerson: Send + Sync
{
    fn name(&self) -> String;
}

struct Person
{
    name: String,
}

impl Person
{
    fn new(name: String) -> Self
    {
        Self { name }
    }
}

impl IPerson for Person
{
    fn name(&self) -> String
    {
        self.name.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()>
{
    let mut di_container = AsyncDIContainer::new();

    di_container.bind::<IFooFactory>().to_async_factory(&|_| {
        Box::new(|cnt| {
            Box::pin(async move {
                let foo_ptr = Box::new(Foo::new(cnt));

                foo_ptr as Box<dyn IFoo>
            })
        })
    })?;

    di_container
        .bind::<dyn IPerson>()
        .to_async_dynamic_value(&|_| {
            Box::new(|| {
                Box::pin(async {
                    // Do some time demanding thing...
                    sleep(Duration::from_secs(1)).await;

                    let person = TransientPtr::new(Person::new("Bob".to_string()));

                    person as TransientPtr<dyn IPerson>
                })
            })
        })?;

    let foo_factory = di_container
        .get::<IFooFactory>()
        .await?
        .threadsafe_factory()?;

    let foo_ptr = foo_factory(4).await;

    foo_ptr.bar();

    let person = di_container.get::<dyn IPerson>().await?.transient()?;

    println!("Person name is {}", person.name());

    Ok(())
}
