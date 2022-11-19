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

pub mod arc;
pub mod boxed;
pub mod error;
pub mod rc;

pub type BoxedCaster = Box<dyn Any + Send + Sync>;

/// A distributed slice gathering constructor functions for [`Caster`]s.
///
/// A constructor function returns `TypeId` of a concrete type involved in the casting
/// and a `Box` of a type or trait backed by a [`Caster`].
#[distributed_slice]
pub static CASTERS: [fn() -> (TypeId, BoxedCaster)] = [..];

/// A `HashMap` mapping `TypeId` of a [`Caster`] to an instance of it.
static CASTER_MAP: Lazy<AHashMap<(TypeId, TypeId), BoxedCaster>> = Lazy::new(|| {
    CASTERS
        .iter()
        .map(|caster_fn| {
            let (type_id, caster) = caster_fn();

            ((type_id, (*caster).type_id()), caster)
        })
        .collect()
});

type CastBoxFn<Dest> = fn(from: Box<dyn Any>) -> Result<Box<Dest>, CasterError>;

type CastRcFn<Dest> = fn(from: Rc<dyn Any>) -> Result<Rc<Dest>, CasterError>;

type CastArcFn<Dest> =
    fn(from: Arc<dyn Any + Sync + Send + 'static>) -> Result<Arc<Dest>, CasterError>;

/// A `Caster` knows how to cast a type or trait to the type or trait `Dest`. Each
/// `Caster` instance is specific to a concrete type. That is, it knows how to cast to
/// single specific type or trait implemented by single specific type.
pub struct Caster<Dest: ?Sized + 'static>
{
    /// Casts a `Box` holding a type or trait object for `Any` to another `Box` holding a
    /// type or trait `Dest`.
    pub cast_box: CastBoxFn<Dest>,

    /// Casts an `Rc` holding a type or trait for `Any` to another `Rc` holding a type or
    /// trait `Dest`.
    pub cast_rc: CastRcFn<Dest>,

    /// Casts an `Arc` holding a type or trait for `Any + Sync + Send + 'static` to
    /// another `Arc` holding a type or trait for `Dest`.
    pub opt_cast_arc: Option<CastArcFn<Dest>>,
}

impl<Dest: ?Sized + 'static> Caster<Dest>
{
    pub fn new(cast_box: CastBoxFn<Dest>, cast_rc: CastRcFn<Dest>) -> Caster<Dest>
    {
        Caster::<Dest> {
            cast_box,
            cast_rc,
            opt_cast_arc: None,
        }
    }

    #[allow(clippy::similar_names)]
    pub fn new_sync(
        cast_box: CastBoxFn<Dest>,
        cast_rc: CastRcFn<Dest>,
        cast_arc: CastArcFn<Dest>,
    ) -> Caster<Dest>
    {
        Caster::<Dest> {
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

/// Returns a `Caster<Dest>` from a concrete type with the id `type_id` to a type or trait
/// `Dest`.
fn get_caster<Dest: ?Sized + 'static>(
    type_id: TypeId,
) -> Result<&'static Caster<Dest>, GetCasterError>
{
    let any_caster = CASTER_MAP
        .get(&(type_id, TypeId::of::<Caster<Dest>>()))
        .ok_or(GetCasterError::NotFound)?;

    any_caster
        .downcast_ref::<Caster<Dest>>()
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

impl<Source: Sized + Any + 'static> CastFrom for Source
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

impl<Source: Sized + Sync + Send + 'static> CastFromSync for Source
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
