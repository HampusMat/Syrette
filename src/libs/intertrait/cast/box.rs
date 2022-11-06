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

use std::any::type_name;

use crate::libs::intertrait::cast::error::CastError;
use crate::libs::intertrait::{get_caster, CastFrom};

pub trait CastBox
{
    /// Casts a box to this trait into that of type `OtherTrait`.
    fn cast<OtherTrait: ?Sized + 'static>(
        self: Box<Self>,
    ) -> Result<Box<OtherTrait>, CastError>;
}

/// A blanket implementation of `CastBox` for traits extending `CastFrom`.
impl<CastFromSelf: ?Sized + CastFrom> CastBox for CastFromSelf
{
    fn cast<OtherTrait: ?Sized + 'static>(
        self: Box<Self>,
    ) -> Result<Box<OtherTrait>, CastError>
    {
        let caster = get_caster::<OtherTrait>((*self).type_id())
            .map_err(CastError::GetCasterFailed)?;

        (caster.cast_box)(self.box_any()).map_err(|err| CastError::CastFailed {
            source: err,
            from: type_name::<Self>(),
            to: type_name::<OtherTrait>(),
        })
    }
}
