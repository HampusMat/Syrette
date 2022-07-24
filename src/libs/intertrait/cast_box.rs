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
use crate::libs::intertrait::{caster, CastFrom};

pub trait CastBox
{
    /// Casts a box to this trait into that of `Trait`. If fails, returns the receiver.
    fn cast<Trait: ?Sized + 'static>(self: Box<Self>) -> Result<Box<Trait>, Box<Self>>;
}

/// A blanket implementation of `CastBox` for traits extending `CastFrom`.
impl<CastableFrom: ?Sized + CastFrom> CastBox for CastableFrom
{
    fn cast<Trait: ?Sized + 'static>(self: Box<Self>) -> Result<Box<Trait>, Box<Self>>
    {
        match caster::<Trait>((*self).type_id()) {
            Some(caster) => Ok((caster.cast_box)(self.box_any())),
            None => Err(self),
        }
    }
}
