#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::disallowed_names)]

use syrette::di_container::blocking::prelude::*;
use syrette::errors::di_container::DIContainerError;
use syrette::errors::injectable::InjectableError;
use syrette::injectable;
use syrette::ptr::TransientPtr;

#[derive(Debug)]
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

#[derive(Debug)]
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

macro_rules! assert_match {
    ($target: expr, $pattern: pat => $expr: expr) => {{
        let target = $target;

        // Not all pattern variables will be used here
        #[allow(unused_variables)]
        {
            assert!(matches!(&target, $pattern));
        }

        match target {
            $pattern => $expr,
            _ => {
                unreachable!();
            }
        }
    }};
}

#[test]
fn prevent_circular_works()
{
    let mut di_container = DIContainer::new();

    di_container.bind::<Foo>().to::<Foo>().expect("Expected Ok");
    di_container.bind::<Bar>().to::<Bar>().expect("Expected Ok");

    let err = di_container.get::<Foo>().expect_err("Expected Err");

    let container_err_a = assert_match!(
        err,
        DIContainerError::BindingResolveFailed {
            reason: InjectableError::ResolveFailed { reason, affected: _ },
            interface: _
        } => *reason
    );

    let container_err_b = assert_match!(
        container_err_a,
        DIContainerError::BindingResolveFailed {
            reason: InjectableError::ResolveFailed { reason, affected: _ },
            interface: _
        } => *reason
    );

    assert!(matches!(
        container_err_b,
        DIContainerError::BindingResolveFailed {
            reason: InjectableError::DetectedCircular {
                dependency_history: _
            },
            interface: _
        }
    ));
}
