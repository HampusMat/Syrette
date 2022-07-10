//! Syrette
//!
//! Syrette is a collection of utilities useful for performing dependency injection.
//!
//! # Examples
//! ```
//! use syrette::errors::di_container::DIContainerError;
//! use syrette::{injectable, DIContainer};
//!
//! trait IDog
//! {
//!     fn woof(&self);
//! }
//!
//! struct Dog {}
//!
//! #[injectable(IDog)]
//! impl Dog
//! {
//!     fn new() -> Self
//!     {
//!         Self {}
//!     }
//! }
//!
//! impl IDog for Dog
//! {
//!     fn woof(&self)
//!     {
//!         println!("Woof!");
//!     }
//! }
//!
//! trait ICat
//! {
//!     fn meow(&self);
//! }
//!
//! struct Cat {}
//!
//! #[injectable(ICat)]
//! impl Cat
//! {
//!     fn new() -> Self
//!     {
//!         Self {}
//!     }
//! }
//!
//! impl ICat for Cat
//! {
//!     fn meow(&self)
//!     {
//!         println!("Meow!");
//!     }
//! }
//!
//! trait IHuman
//! {
//!     fn make_pets_make_sounds(&self);
//! }
//!
//! struct Human
//! {
//!     _dog: Box<dyn IDog>,
//!     _cat: Box<dyn ICat>,
//! }
//!
//! #[injectable(IHuman)]
//! impl Human
//! {
//!     fn new(dog: Box<dyn IDog>, cat: Box<dyn ICat>) -> Self
//!     {
//!         Self {
//!             _dog: dog,
//!             _cat: cat,
//!         }
//!     }
//! }
//!
//! impl IHuman for Human
//! {
//!     fn make_pets_make_sounds(&self)
//!     {
//!         println!("Hi doggy!");
//!
//!         self._dog.woof();
//!
//!         println!("Hi kitty!");
//!
//!         self._cat.meow();
//!     }
//! }
//!
//! fn main() -> error_stack::Result<(), DIContainerError>
//! {
//!     println!("Hello, world!");
//!
//!     let mut di_container: DIContainer = DIContainer::new();
//!
//!     di_container.bind::<dyn IDog>().to::<Dog>();
//!     di_container.bind::<dyn ICat>().to::<Cat>();
//!     di_container.bind::<dyn IHuman>().to::<Human>();
//!
//!     let dog = di_container.get::<dyn IDog>()?;
//!
//!     dog.woof();
//!
//!     let human = di_container.get::<dyn IHuman>()?;
//!
//!     human.make_pets_make_sounds();
//!
//!     Ok(())
//! }
//!
//! ```

pub mod di_container;
pub mod errors;
pub mod interfaces;

pub use di_container::*;
pub use syrette_macros::*;

#[doc(hidden)]
pub mod libs;

// Private
mod provider;
