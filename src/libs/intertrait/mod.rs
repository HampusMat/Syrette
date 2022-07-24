/**
 * Originally from Intertrait by CodeChain
 *
 * <https://github.com/CodeChain-io/intertrait>
 * <https://crates.io/crates/intertrait/0.2.2>
 *
 * Licensed under either of
 *
 * Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

 * at your option.
*/
use std::any::{Any, TypeId};
use std::rc::Rc;

use ahash::AHashMap;
use linkme::distributed_slice;
use once_cell::sync::Lazy;

pub mod cast_box;
pub mod cast_rc;

pub type BoxedCaster = Box<dyn Any + Send + Sync>;

type CasterFn = fn() -> (TypeId, BoxedCaster);

#[distributed_slice]
pub static CASTERS: [CasterFn] = [..];

static CASTER_MAP: Lazy<AHashMap<(TypeId, TypeId), BoxedCaster>> = Lazy::new(|| {
    CASTERS
        .iter()
        .map(|caster_fn| {
            let (type_id, caster) = caster_fn();

            ((type_id, (*caster).type_id()), caster)
        })
        .collect()
});

pub struct Caster<Trait: ?Sized + 'static>
{
    /// Casts a `Box` holding a trait object for `Any` to another `Box` holding a trait object
    /// for `Trait`.
    pub cast_box: fn(from: Box<dyn Any>) -> Box<Trait>,

    /// Casts an `Rc` holding a trait object for `Any` to another `Rc` holding a trait object
    /// for `Trait`.
    pub cast_rc: fn(from: Rc<dyn Any>) -> Rc<Trait>,
}

impl<Trait: ?Sized + 'static> Caster<Trait>
{
    pub fn new(
        cast_box: fn(from: Box<dyn Any>) -> Box<Trait>,
        cast_rc: fn(from: Rc<dyn Any>) -> Rc<Trait>,
    ) -> Caster<Trait>
    {
        Caster::<Trait> { cast_box, cast_rc }
    }
}

/// Returns a `Caster<Implementation, Trait>` from a concrete type `Implementation`
/// from inside `CASTER_MAP` to a `Trait` implemented by it.
fn caster<Trait: ?Sized + 'static>(type_id: TypeId) -> Option<&'static Caster<Trait>>
{
    CASTER_MAP
        .get(&(type_id, TypeId::of::<Caster<Trait>>()))
        .and_then(|caster| caster.downcast_ref::<Caster<Trait>>())
}

/// `CastFrom` must be extended by a trait that wants to allow for casting into another trait.
///
/// It is used for obtaining a trait object for [`Any`] from a trait object for its sub-trait,
/// and blanket implemented for all `Sized + Any + 'static` types.
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
