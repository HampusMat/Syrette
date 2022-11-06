//! A library providing direct casting among trait objects implemented by a type.
//!
//! In Rust, an object of a sub-trait of [`Any`] can be downcast to a concrete type
//! at runtime if the type is known. But no direct casting between two trait objects
//! (i.e. without involving the concrete type of the backing value) is possible
//! (even no coercion from a trait object to that of its super-trait yet).
//!
//! With this crate, any trait object with [`CastFrom`] as its super-trait can be cast
//! directly to another trait object implemented by the underlying type if the target
//! traits are registered beforehand with the macros provided by this crate.
//!
//!
//! Originally from Intertrait by CodeChain
//!
//! <https://github.com/CodeChain-io/intertrait>
//! <https://crates.io/crates/intertrait/0.2.2>
//!
//! Licensed under either of
//!
//! Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
//! MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
use std::any::{Any, TypeId};
use std::rc::Rc;
use std::sync::Arc;

use ahash::AHashMap;
use linkme::distributed_slice;
use once_cell::sync::Lazy;

pub mod cast;

pub type BoxedCaster = Box<dyn Any + Send + Sync>;

/// A distributed slice gathering constructor functions for [`Caster<T>`]s.
///
/// A constructor function returns `TypeId` of a concrete type involved in the casting
/// and a `Box` of a trait object backed by a [`Caster<T>`].
///
/// [`Caster<T>`]: ./struct.Caster.html
#[distributed_slice]
pub static CASTERS: [fn() -> (TypeId, BoxedCaster)] = [..];

/// A `HashMap` mapping `TypeId` of a [`Caster<T>`] to an instance of it.
///
/// [`Caster<T>`]: ./struct.Caster.html
static CASTER_MAP: Lazy<AHashMap<(TypeId, TypeId), BoxedCaster>> = Lazy::new(|| {
    CASTERS
        .iter()
        .map(|caster_fn| {
            let (type_id, caster) = caster_fn();

            ((type_id, (*caster).type_id()), caster)
        })
        .collect()
});

type CastBoxFn<Trait> = fn(from: Box<dyn Any>) -> Result<Box<Trait>, CasterError>;

type CastRcFn<Trait> = fn(from: Rc<dyn Any>) -> Result<Rc<Trait>, CasterError>;

type CastArcFn<Trait> =
    fn(from: Arc<dyn Any + Sync + Send + 'static>) -> Result<Arc<Trait>, CasterError>;

/// A `Caster` knows how to cast a reference to or `Box` of a trait object for `Any`
/// to a trait object of trait `Trait`. Each `Caster` instance is specific to a concrete
/// type. That is, it knows how to cast to single specific trait implemented by single
/// specific type.
pub struct Caster<Trait: ?Sized + 'static>
{
    /// Casts a `Box` holding a trait object for `Any` to another `Box` holding a trait
    /// object for trait `Trait`.
    pub cast_box: CastBoxFn<Trait>,

    /// Casts an `Rc` holding a trait object for `Any` to another `Rc` holding a trait
    /// object for trait `Trait`.
    pub cast_rc: CastRcFn<Trait>,

    /// Casts an `Arc` holding a trait object for `Any + Sync + Send + 'static`
    /// to another `Arc` holding a trait object for trait `Trait`.
    pub opt_cast_arc: Option<CastArcFn<Trait>>,
}

impl<Trait: ?Sized + 'static> Caster<Trait>
{
    pub fn new(cast_box: CastBoxFn<Trait>, cast_rc: CastRcFn<Trait>) -> Caster<Trait>
    {
        Caster::<Trait> {
            cast_box,
            cast_rc,
            opt_cast_arc: None,
        }
    }

    #[allow(clippy::similar_names)]
    pub fn new_sync(
        cast_box: CastBoxFn<Trait>,
        cast_rc: CastRcFn<Trait>,
        cast_arc: CastArcFn<Trait>,
    ) -> Caster<Trait>
    {
        Caster::<Trait> {
            cast_box,
            cast_rc,
            opt_cast_arc: Some(cast_arc),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CasterError
{
    #[error("Failed to cast Box")]
    CastBoxFailed,

    #[error("Failed to cast Rc")]
    CastRcFailed,

    #[error("Failed to cast Arc")]
    CastArcFailed,
}

/// Returns a `Caster<S, Trait>` from a concrete type `S` to a trait `Trait` implemented
/// by it.
fn get_caster<Trait: ?Sized + 'static>(
    type_id: TypeId,
) -> Result<&'static Caster<Trait>, GetCasterError>
{
    let any_caster = CASTER_MAP
        .get(&(type_id, TypeId::of::<Caster<Trait>>()))
        .ok_or(GetCasterError::NotFound)?;

    any_caster
        .downcast_ref::<Caster<Trait>>()
        .ok_or(GetCasterError::DowncastFailed)
}

#[derive(Debug, thiserror::Error)]
pub enum GetCasterError
{
    #[error("Caster not found")]
    NotFound,

    #[error("Failed to downcast caster")]
    DowncastFailed,
}

/// `CastFrom` must be extended by a trait that wants to allow for casting into another
/// trait.
///
/// It is used for obtaining a trait object for [`Any`] from a trait object for its
/// sub-trait, and blanket implemented for all `Sized + Any + 'static` types.
///
/// # Examples
/// ```ignore
/// trait Source: CastFrom {
///     ...
/// }
/// ```
pub trait CastFrom: Any + 'static
{
    /// Returns a `Box` of `Any`, which is backed by the type implementing this trait.
    fn box_any(self: Box<Self>) -> Box<dyn Any>;

    /// Returns an `Rc` of `Any`, which is backed by the type implementing this trait.
    fn rc_any(self: Rc<Self>) -> Rc<dyn Any>;
}

/// `CastFromSync` must be extended by a trait that is `Any + Sync + Send + 'static`
/// and wants to allow for casting into another trait behind references and smart pointers
/// especially including `Arc`.
///
/// It is used for obtaining a trait object for [`Any + Sync + Send + 'static`] from an
/// object for its sub-trait, and blanket implemented for all `Sized + Sync + Send +
/// 'static` types.
///
/// # Examples
/// ```ignore
/// trait Source: CastFromSync {
///     ...
/// }
/// ```
pub trait CastFromSync: CastFrom + Sync + Send + 'static
{
    fn arc_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send + 'static>;
}

impl<Trait: Sized + Any + 'static> CastFrom for Trait
{
    fn box_any(self: Box<Self>) -> Box<dyn Any>
    {
        self
    }

    fn rc_any(self: Rc<Self>) -> Rc<dyn Any>
    {
        self
    }
}

impl CastFrom for dyn Any + 'static
{
    fn box_any(self: Box<Self>) -> Box<dyn Any>
    {
        self
    }

    fn rc_any(self: Rc<Self>) -> Rc<dyn Any>
    {
        self
    }
}

impl<Trait: Sized + Sync + Send + 'static> CastFromSync for Trait
{
    fn arc_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send + 'static>
    {
        self
    }
}

impl CastFrom for dyn Any + Sync + Send + 'static
{
    fn box_any(self: Box<Self>) -> Box<dyn Any>
    {
        self
    }

    fn rc_any(self: Rc<Self>) -> Rc<dyn Any>
    {
        self
    }
}

impl CastFromSync for dyn Any + Sync + Send + 'static
{
    fn arc_any(self: Arc<Self>) -> Arc<dyn Any + Sync + Send + 'static>
    {
        self
    }
}

#[cfg(test)]
mod tests
{
    use std::any::TypeId;
    use std::fmt::Debug;

    use linkme::distributed_slice;

    use super::*;
    use crate::test_utils::subjects;

    #[distributed_slice(super::CASTERS)]
    static TEST_CASTER: fn() -> (TypeId, BoxedCaster) = create_test_caster;

    fn create_test_caster() -> (TypeId, BoxedCaster)
    {
        let type_id = TypeId::of::<subjects::Ninja>();

        let caster = Box::new(Caster::<dyn Debug> {
            cast_box: |from| {
                let concrete = from
                    .downcast::<subjects::Ninja>()
                    .map_err(|_| CasterError::CastBoxFailed)?;

                Ok(concrete as Box<dyn Debug>)
            },
            cast_rc: |from| {
                let concrete = from
                    .downcast::<subjects::Ninja>()
                    .map_err(|_| CasterError::CastRcFailed)?;

                Ok(concrete as Rc<dyn Debug>)
            },
            opt_cast_arc: Some(|from| {
                let concrete = from
                    .downcast::<subjects::Ninja>()
                    .map_err(|_| CasterError::CastArcFailed)?;

                Ok(concrete as Arc<dyn Debug>)
            }),
        });
        (type_id, caster)
    }
}
