## Syrette
[![Latest Version](https://img.shields.io/crates/v/syrette)](https://crates.io/crates/syrette)
[![Documentation](https://img.shields.io/badge/docs.rs-syrette-brightgreen)](https://docs.rs/syrette)

The convenient dependency injection library for Rust.

## Namesake
From the [syrette Wikipedia article](https://en.wikipedia.org/wiki/Syrette).
> A syrette is a device for injecting liquid through a needle.
> It is similar to a syringe except that it has a closed flexible
> tube (like that typically used for toothpaste) instead of a rigid tube and piston.

## Features
- A [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) container
- Autowiring dependencies
- Binding factories
- API inspired from the one of [InversifyJS](https://github.com/inversify/InversifyJS)
- Helpful error messages
- Enforces the use of interface traits

## Motivation
Other DI libraries for Rust are either unmaintained ([di](https://crates.io/crates/di) for example),
overcomplicated and or bloated ([anthill-di](https://crates.io/crates/anthill-di) for example)
or has a weird API ([teloc](https://crates.io/crates/teloc) for example).

The goal of Syrette is to be a simple, useful, convenient and familiar DI library.

## Notice
Rust nightly is currently required.

## Example usage
```rust
use syrette::{injectable, DIContainer};
use syrette::ptr::InterfacePtr;

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

struct Warrior {
	weapon: InterfacePtr<dyn IWeapon>,
}

#[injectable(IWarrior)]
impl Warrior
{
	fn new(weapon: InterfacePtr<dyn IWeapon>) -> Self
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

fn main()
{
	let mut di_container = DIContainer::new();

	di_container.bind::<dyn IWeapon>().to::<Sword>();

	di_container.bind::<dyn IWarrior>().to::<Warrior>();

	let warrior = di_container.get::<dyn IWarrior>().unwrap();

	warrior.fight();

	println!("Warrio has fighted");
}
```

For more examples see the [examples folder](https://git.hampusmat.com/syrette/tree/examples).

## Todo
- Add support for generics

