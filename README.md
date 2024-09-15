## Syrette
[![Latest Version](https://img.shields.io/crates/v/syrette)](https://crates.io/crates/syrette)
[![Documentation](https://img.shields.io/badge/docs.rs-syrette-blueviolet)](https://docs.rs/syrette)
[![Build](https://img.shields.io/circleci/build/github/HampusMat/Syrette/master)](https://app.circleci.com/pipelines/github/HampusMat/Syrette)
[![Coverage](https://img.shields.io/codecov/c/github/HampusMat/Syrette)](https://app.codecov.io/gh/HampusMat/Syrette)
[![Rust](https://img.shields.io/badge/rust-1.62.1%2B-informational)](#rust-version-requirements)

The convenient dependency injection & inversion of control framework for Rust.

<div class="warning">

Currently, This crate does not work on nightly versions of Rust. This is because Nightly
versions uses rust-lld. See [Linkme issue #94](https://github.com/dtolnay/linkme/issues/94).

A temporary fix for this is to build with `RUSTFLAGS="-C link-args=-znostart-stop-gc"`
</div>

## Namesake
From the [syrette Wikipedia article](https://en.wikipedia.org/wiki/Syrette).
> A syrette is a device for injecting liquid through a needle.
> It is similar to a syringe except that it has a closed flexible
> tube (like that typically used for toothpaste) instead of a rigid tube and piston.

## Features
- A [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) and [inversion of control](https://en.wikipedia.org/wiki/Inversion_of_control) container
- Autowiring dependencies
- API inspired from the one of [InversifyJS](https://github.com/inversify/InversifyJS)
- Helpful error messages
- Supports generic implementations & generic interface traits
- Binding singletons
- Injection of third-party structs & traits
- Named bindings
- Async factories

## Optional features
- `factory`. Binding factories (Rust nightly required)
- `prevent-circular`. Detection and prevention of circular dependencies. (Enabled by default)
- `async`. Asynchronous support

To use these features, you must [enable it in Cargo](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features).

## Why inversion of control & dependency injection?
The reason for practing IoC and DI is to write modular & loosely coupled applications.

This is what we're trying to avoid:
```rust
impl Foo
{
    /// ❌ Bad. Foo knows the construction details of Bar.
    pub fn new() -> Self
    {
        Self {
            bar: Bar::new()
        }
    }
```

The following is better:
```rust
impl Foo
    /// ✅ Better. Foo is unaware of how Bar is constructed.
    pub fn new(bar: Bar) -> Self
    {
        Self {
            bar
        }
    }
}
```

This will however grow quite tiresome sooner or later when you have a large codebase
with many dependencies and dependencies of those and so on. Because you will have to
specify the dependencies someplace

```rust
let foobar = Foobar::new(
    Foo:new(
        Woof::new(),
        Meow::new()),
    Bar::new(
        Something::new(),
        SomethingElse::new(),
        SomethingMore::new()
    )
)
```

This is where Syrette comes in.

## Motivation
Other DI & IoC libraries for Rust are either unmaintained ([di](https://crates.io/crates/di) for example),
overcomplicated and requires Rust nightly for all functionality ([anthill-di](https://crates.io/crates/anthill-di) for example)
or has a weird API ([teloc](https://crates.io/crates/teloc) for example).

The goal of Syrette is to be a simple, useful, convenient and familiar DI & IoC library.

## Example usage
```rust
use std::error::Error;

use syrette::injectable;
use syrette::DIContainer;
use syrette::ptr::TransientPtr;

trait IWeapon
{
	fn deal_damage(&self, damage: i32);
}

struct Sword {}

#[injectable(IWeapon)]
impl Sword
{
	fn new() -> Self
	{
		Self {}
	}
}

impl IWeapon for Sword
{
	fn deal_damage(&self, damage: i32)
	{
		println!("Sword dealt {} damage!", damage);
	}
}

trait IWarrior
{
	fn fight(&self);
}

struct Warrior
{
	weapon: TransientPtr<dyn IWeapon>,
}

#[injectable(IWarrior)]
impl Warrior
{
	fn new(weapon: TransientPtr<dyn IWeapon>) -> Self
	{
		Self { weapon }
	}
}

impl IWarrior for Warrior
{
	fn fight(&self)
	{
		self.weapon.deal_damage(30);
	}
}

fn main() -> Result<(), Box<dyn Error>>
{
	let mut di_container = DIContainer::new();

	di_container.bind::<dyn IWeapon>().to::<Sword>()?;

	di_container.bind::<dyn IWarrior>().to::<Warrior>()?;

	let warrior = di_container.get::<dyn IWarrior>()?.transient()?;

	warrior.fight();

	println!("Warrior has fighted");

	Ok(())
}
```

For more examples see the [examples folder](https://git.hampusmat.com/syrette/tree/examples).

## Terminology
**Transient**<br>
A type or trait that is unique to owner.

**Singleton**<br>
A type that only has a single instance. The opposite of transient. Generally discouraged.

**Interface**<br>
A type or trait that represents a type (itself in the case of it being a type).

**Factory**<br>
A function that creates new instances of a specific type or trait.

## Rust version requirements
Syrette requires Rust >= 1.62.1 to work. This is mainly due to the dependency on [Linkme](https://crates.io/crates/linkme).

## Todo
- Add support for generic factories

## Contributing
You can reach out by joining the [mailing list](https://lists.hampusmat.com/postorius/lists/syrette.lists.hampusmat.com/).

This is the place to submit patches, feature requests and to report bugs.

