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

type CastArcFn<Trait> = fn(from: Arc<dyn Any + Sync + Send + 'static>) -> Arc<Trait>;

/// A `Caster` knows how to cast a reference to or `Box` of a trait object for `Any`
/// to a trait object of trait `Trait`. Each `Caster` instance is specific to a concrete
/// type. That is, it knows how to cast to single specific trait implemented by single
/// specific type.
pub struct Caster<Trait: ?Sized + 'static>
{
    /// Casts a `Box` holding a trait object for `Any` to another `Box` holding a trait
    /// object for trait `Trait`.
    pub cast_box: fn(from: Box<dyn Any>) -> Box<Trait>,

    /// Casts an `Rc` holding a trait object for `Any` to another `Rc` holding a trait
    /// object for trait `Trait`.
    pub cast_rc: fn(from: Rc<dyn Any>) -> Rc<Trait>,

    /// Casts an `Arc` holding a trait object for `Any + Sync + Send + 'static`
    /// to another `Arc` holding a trait object for trait `Trait`.
    pub opt_cast_arc: Option<CastArcFn<Trait>>,
}

impl<Trait: ?Sized + 'static> Caster<Trait>
{
    pub fn new(
        cast_box: fn(from: Box<dyn Any>) -> Box<Trait>,
        cast_rc: fn(from: Rc<dyn Any>) -> Rc<Trait>,
    ) -> Caster<Trait>
    {
        Caster::<Trait> {
            cast_box,
            cast_rc,
            opt_cast_arc: None,
        }
    }

    #[allow(clippy::similar_names)]
    pub fn new_sync(
        cast_box: fn(from: Box<dyn Any>) -> Box<Trait>,
        cast_rc: fn(from: Rc<dyn Any>) -> Rc<Trait>,
        cast_arc: fn(from: Arc<dyn Any + Sync + Send>) -> Arc<Trait>,
    ) -> Caster<Trait>
    {
        Caster::<Trait> {
            cast_box,
            cast_rc,
            opt_cast_arc: Some(cast_arc),
        }
    }
}

/// Returns a `Caster<S, Trait>` from a concrete type `S` to a trait `Trait` implemented
/// by it.
fn get_caster<Trait: ?Sized + 'static>(type_id: TypeId)
    -> Option<&'static Caster<Trait>>
{
    CASTER_MAP
        .get(&(type_id, TypeId::of::<Caster<Trait>>()))
        .and_then(|caster| caster.downcast_ref::<Caster<Trait>>())
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
    use std::any::{Any, TypeId};
    use std::fmt::{Debug, Display};

    use linkme::distributed_slice;

    #[allow(clippy::wildcard_imports)]
    use super::cast::*;
    use super::*;

    #[distributed_slice(super::CASTERS)]
    static TEST_CASTER: fn() -> (TypeId, BoxedCaster) = create_test_caster;

    #[derive(Debug)]
    struct TestStruct;

    trait SourceTrait: CastFromSync {}

    impl SourceTrait for TestStruct {}

    fn create_test_caster() -> (TypeId, BoxedCaster)
    {
        let type_id = TypeId::of::<TestStruct>();
        let caster = Box::new(Caster::<dyn Debug> {
            cast_box: |from| from.downcast::<TestStruct>().unwrap(),
            cast_rc: |from| from.downcast::<TestStruct>().unwrap(),
            opt_cast_arc: Some(|from| from.downcast::<TestStruct>().unwrap()),
        });
        (type_id, caster)
    }

    #[test]
    fn cast_box()
    {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn cast_rc()
    {
        let ts = Rc::new(TestStruct);
        let st: Rc<dyn SourceTrait> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn cast_arc()
    {
        let ts = Arc::new(TestStruct);
        let st: Arc<dyn SourceTrait> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn cast_box_wrong()
    {
        let ts = Box::new(TestStruct);
        let st: Box<dyn SourceTrait> = ts;
        let display = st.cast::<dyn Display>();
        assert!(display.is_err());
    }

    #[test]
    fn cast_rc_wrong()
    {
        let ts = Rc::new(TestStruct);
        let st: Rc<dyn SourceTrait> = ts;
        let display = st.cast::<dyn Display>();
        assert!(display.is_err());
    }

    #[test]
    fn cast_arc_wrong()
    {
        let ts = Arc::new(TestStruct);
        let st: Arc<dyn SourceTrait> = ts;
        let display = st.cast::<dyn Display>();
        assert!(display.is_err());
    }

    #[test]
    fn cast_box_from_any()
    {
        let ts = Box::new(TestStruct);
        let st: Box<dyn Any> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn cast_rc_from_any()
    {
        let ts = Rc::new(TestStruct);
        let st: Rc<dyn Any> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }

    #[test]
    fn cast_arc_from_any()
    {
        let ts = Arc::new(TestStruct);
        let st: Arc<dyn Any + Send + Sync> = ts;
        let debug = st.cast::<dyn Debug>();
        assert!(debug.is_ok());
    }
}
