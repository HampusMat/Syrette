#![cfg_attr(feature = "factory", feature(unboxed_closures, tuple_trait))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![deny(missing_docs)]

//! Syrette
//!
//! Syrette is a framework for utilizing inversion of control & dependency injection.
//!
//! # Example
//! ```
//! use std::error::Error;
//!
//! use syrette::ptr::TransientPtr;
//! use syrette::{injectable, DIContainer};
//!
//! trait IWeapon
//! {
//!     fn deal_damage(&self, damage: i32);
//! }
//!
//! struct Sword {}
//!
//! #[injectable(IWeapon)] // Makes Sword injectable with a interface of IWeapon
//! impl Sword
//! {
//!     fn new() -> Self
//!     {
//!         Self {}
//!     }
//! }
//!
//! impl IWeapon for Sword
//! {
//!     fn deal_damage(&self, damage: i32)
//!     {
//!         println!("Sword dealt {} damage!", damage);
//!     }
//! }
//!
//! trait IWarrior
//! {
//!     fn fight(&self);
//! }
//!
//! struct Warrior
//! {
//!     weapon: TransientPtr<dyn IWeapon>,
//! }
//!
//! #[injectable(IWarrior)] // Makes Warrior injectable with a interface of IWarrior
//! impl Warrior
//! {
//!     fn new(weapon: TransientPtr<dyn IWeapon>) -> Self
//!     {
//!         Self { weapon }
//!     }
//! }
//!
//! impl IWarrior for Warrior
//! {
//!     fn fight(&self)
//!     {
//!         self.weapon.deal_damage(30);
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn Error>>
//! {
//!     let mut di_container = DIContainer::new();
//!
//!     // Creates a binding of the interface IWeapon to the concrete type Sword
//!     di_container.bind::<dyn IWeapon>().to::<Sword>()?;
//!
//!     // Creates a binding of the interface IWarrior to the concrete type Warrior
//!     di_container.bind::<dyn IWarrior>().to::<Warrior>()?;
//!
//!     // Create a transient IWarrior with all of its dependencies automatically injected
//!     let warrior = di_container.get::<dyn IWarrior>()?.transient()?;
//!
//!     warrior.fight();
//!
//!     println!("Warrior has fighted");
//!
//!     Ok(())
//! }
//! ```

pub mod dependency_history;
pub mod di_container;
pub mod errors;
pub mod interfaces;
pub mod ptr;
pub mod ptr_buffer;

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub mod future;

#[cfg(feature = "async")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "async")))]
pub use di_container::asynchronous::AsyncDIContainer;
pub use di_container::blocking::DIContainer;
pub use syrette_macros::{injectable, named};

mod provider;
mod util;

#[cfg(feature = "factory")]
mod castable_factory;

#[cfg(feature = "factory")]
mod any_factory;

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod test_utils;

/// Shortcut for creating a DI container binding for a injectable without a declared
/// interface.
///
/// This will declare a interface for the implementation.
///
/// Useful for when the implementation or the interface is generic.
///
/// # Arguments
/// {interface} => {implementation}, {DI container variable name}
///
/// # Examples
/// ```
/// # use syrette::{di_container_bind, DIContainer, injectable};
/// #
/// # trait INinja {}
/// #
/// # struct Ninja {}
/// #
/// # #[injectable]
/// # impl Ninja
/// # {
/// #     fn new() -> Self
/// #     {
/// #         Self {}
/// #     }
/// # }
/// #
/// # impl INinja for Ninja {}
/// #
/// let mut di_container = DIContainer::new();
///
/// di_container_bind!(INinja => Ninja, di_container);
/// ```
#[cfg(not(tarpaulin_include))]
#[macro_export]
macro_rules! di_container_bind {
    ($interface: path => $implementation: ty, $di_container: ident) => {
        $di_container
            .bind::<dyn $interface>()
            .to::<$implementation>()
            .unwrap();
    };
}
