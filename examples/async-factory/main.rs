#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use anyhow::Result;
use syrette::{async_closure, factory, AsyncDIContainer};

trait IFoo
{
    fn bar(&self);
}

#[factory(async = true)]
type IFooFactory = dyn Fn(i32) -> dyn IFoo;

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

#[tokio::main]
async fn main() -> Result<()>
{
    let mut di_container = AsyncDIContainer::new();

    di_container
        .bind::<IFooFactory>()
        .to_async_factory(&|_| {
            async_closure!(|cnt| {
                let foo = Box::new(Foo::new(cnt));

                foo as Box<dyn IFoo>
            })
        })
        .await?;

    let foo_factory = di_container
        .get::<IFooFactory>()
        .await?
        .threadsafe_factory()?;

    let foo = foo_factory(4).await;

    foo.bar();

    Ok(())
}
