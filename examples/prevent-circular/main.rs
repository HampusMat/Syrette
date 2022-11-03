//! Example demonstrating the prevention of circular dependencies.
//!
//! Having circular dependencies is generally bad practice and is detected by Syrette when
//! the `prevent-circular` feature is enabled.
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::disallowed_names)]

use syrette::di_container::blocking::prelude::*;
use syrette::injectable;
use syrette::ptr::TransientPtr;

struct Foo
{
    _bar: TransientPtr<Bar>,
}

#[injectable]
impl Foo
{
    fn new(bar: TransientPtr<Bar>) -> Self
    {
        Self { _bar: bar }
    }
}

struct Bar
{
    _foo: TransientPtr<Foo>,
}

#[injectable]
impl Bar
{
    fn new(foo: TransientPtr<Foo>) -> Self
    {
        Self { _foo: foo }
    }
}

fn main() -> Result<(), anyhow::Error>
{
    let mut di_container = DIContainer::new();

    di_container.bind::<Foo>().to::<Foo>()?;
    di_container.bind::<Bar>().to::<Bar>()?;

    // The following won't work. Err will be returned.
    let _foo = di_container.get::<Foo>()?.transient()?;

    Ok(())
}
