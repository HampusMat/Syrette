#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::disallowed_names)]
use std::error::Error;

use syrette::di_container::blocking::prelude::*;
use syrette::injectable;
use syrette::ptr::TransientPtr;

struct Foo
{
    bar: TransientPtr<Bar>,
}

#[injectable]
impl Foo
{
    fn new(bar: TransientPtr<Bar>) -> Self
    {
        Self { bar }
    }
}

struct Bar
{
    foo: TransientPtr<Foo>,
}

#[injectable]
impl Bar
{
    fn new(foo: TransientPtr<Foo>) -> Self
    {
        Self { foo }
    }
}

fn main() -> Result<(), anyhow::Error>
{
    let mut di_container = DIContainer::new();

    di_container.bind::<Foo>().to::<Foo>()?;
    di_container.bind::<Bar>().to::<Bar>()?;

    let foo = di_container.get::<Foo>()?.transient()?;

    Ok(())
}
