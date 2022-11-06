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
use std::sync::Arc;

use crate::libs::intertrait::cast::error::CastError;
use crate::libs::intertrait::{get_caster, CastFromSync};

pub trait CastArc
{
    /// Casts an `Arc` for this trait into that for type `OtherTrait`.
    fn cast<OtherTrait: ?Sized + 'static>(
        self: Arc<Self>,
    ) -> Result<Arc<OtherTrait>, CastError>;
}

/// A blanket implementation of `CastArc` for traits extending `CastFrom`, `Sync`, and
/// `Send`.
impl<CastFromSelf: ?Sized + CastFromSync> CastArc for CastFromSelf
{
    fn cast<OtherTrait: ?Sized + 'static>(
        self: Arc<Self>,
    ) -> Result<Arc<OtherTrait>, CastError>
    {
        let caster = get_caster::<OtherTrait>((*self).type_id())
            .map_err(CastError::GetCasterFailed)?;

        let cast_arc = caster
            .opt_cast_arc
            .ok_or(CastError::NotArcCastable(type_name::<OtherTrait>()))?;

        cast_arc(self.arc_any()).map_err(|err| CastError::CastFailed {
            source: err,
            from: type_name::<Self>(),
            to: type_name::<OtherTrait>(),
        })
    }
}
