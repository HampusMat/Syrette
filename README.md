## Syrette
[![Latest Version](https://img.shields.io/crates/v/syrette)](https://crates.io/crates/syrette)
[![Documentation](https://img.shields.io/badge/docs.rs-syrette-blueviolet)](https://docs.rs/syrette)
[![Build](https://img.shields.io/circleci/build/github/HampusMat/Syrette/master)](https://app.circleci.com/pipelines/github/HampusMat/Syrette)


The convenient dependency injection framework for Rust.

## Namesake
From the [syrette Wikipedia article](https://en.wikipedia.org/wiki/Syrette).
> A syrette is a device for injecting liquid through a needle.
> It is similar to a syringe except that it has a closed flexible
> tube (like that typically used for toothpaste) instead of a rigid tube and piston.

## Features
- A [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) container
- Autowiring dependencies
- API inspired from the one of [InversifyJS](https://github.com/inversify/InversifyJS)
- Helpful error messages
- Enforces the use of interface traits
- Supports generic implementations & generic interface traits
- Binding singletons
- Injection of third-party structs & traits
- Named bindings

## Optional features
- `factory`. Binding factories (Rust nightly required)
- `prevent-circular`. Detection and prevention of circular dependencies. (Enabled by default)

To use these features, you must [enable it in Cargo](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features).

## Motivation
Other DI/IOC libraries for Rust are either unmaintained ([di](https://crates.io/crates/di) for example),
overcomplicated and or bloated ([anthill-di](https://crates.io/crates/anthill-di) for example)
or has a weird API ([teloc](https://crates.io/crates/teloc) for example).

The goal of Syrette is to be a simple, useful, convenient and familiar DI library.

## Example usage
```rust
use std::error::Error;

use syrette::{injectable, DIContainer};
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

## Todo
- Add support for generic factories
- Add asynchronous functionality

## Contributing
You can reach out by joining the [mailing list](https://lists.hampusmat.com/postorius/lists/syrette.lists.hampusmat.com/).

This is the place to submit patches, feature requests and to report bugs.

