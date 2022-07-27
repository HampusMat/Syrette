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
use std::rc::Rc;

use crate::libs::intertrait::{caster, CastFrom};

pub trait CastRc
{
    /// Casts an `Rc` for this trait into that for type `T`.
    fn cast<T: ?Sized + 'static>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>>;
}

/// A blanket implementation of `CastRc` for traits extending `CastFrom`.
impl<S: ?Sized + CastFrom> CastRc for S
{
    fn cast<T: ?Sized + 'static>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>>
    {
        match caster::<T>((*self).type_id()) {
            Some(caster) => Ok((caster.cast_rc)(self.rc_any())),
            None => Err(self),
        }
    }
}
