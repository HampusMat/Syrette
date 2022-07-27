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
use std::sync::Arc;

use crate::libs::intertrait::{caster, CastFromSync};

pub trait CastArc
{
    /// Casts an `Arc` for this trait into that for type `T`.
    fn cast<T: ?Sized + 'static>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>>;
}

/// A blanket implementation of `CastArc` for traits extending `CastFrom`, `Sync`, and `Send`.
impl<S: ?Sized + CastFromSync> CastArc for S
{
    fn cast<T: ?Sized + 'static>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>>
    {
        match caster::<T>((*self).type_id()) {
            Some(caster) => Ok((caster.cast_arc)(self.arc_any())),
            None => Err(self),
        }
    }
}
