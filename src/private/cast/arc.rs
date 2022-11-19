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

use crate::private::cast::error::CastError;
use crate::private::cast::{get_caster, CastFromSync};

pub trait CastArc
{
    /// Casts an `Arc` with `Self` into an `Arc` with `Dest`.
    fn cast<Dest: ?Sized + 'static>(self: Arc<Self>) -> Result<Arc<Dest>, CastError>;
}

/// A blanket implementation of `CastArc` for traits extending `CastFrom`, `Sync`, and
/// `Send`.
impl<CastFromSelf: ?Sized + CastFromSync> CastArc for CastFromSelf
{
    fn cast<Dest: ?Sized + 'static>(self: Arc<Self>) -> Result<Arc<Dest>, CastError>
    {
        let caster =
            get_caster::<Dest>((*self).type_id()).map_err(CastError::GetCasterFailed)?;

        let cast_arc = caster
            .opt_cast_arc
            .ok_or(CastError::NotArcCastable(type_name::<Dest>()))?;

        cast_arc(self.arc_any()).map_err(|err| CastError::CastFailed {
            source: err,
            from: type_name::<Self>(),
            to: type_name::<Dest>(),
        })
    }
}

#[cfg(test)]
mod tests
{
    use std::any::Any;
    use std::fmt::{Debug, Display};
    use std::sync::Arc;

    use super::*;
    use crate::test_utils::subjects;

    #[test]
    fn can_cast_arc()
    {
        let concrete_ninja = Arc::new(subjects::Ninja);

        let abstract_ninja: Arc<dyn subjects::INinja> = concrete_ninja;

        let debug_ninja_result = abstract_ninja.cast::<dyn Debug>();

        assert!(debug_ninja_result.is_ok());
    }

    #[test]
    fn cannot_cast_arc_wrong()
    {
        let concrete_ninja = Arc::new(subjects::Ninja);

        let abstract_ninja: Arc<dyn subjects::INinja> = concrete_ninja;

        let display_ninja_result = abstract_ninja.cast::<dyn Display>();

        assert!(matches!(
            display_ninja_result,
            Err(CastError::GetCasterFailed(_))
        ));
    }

    #[test]
    fn can_cast_arc_from_any()
    {
        let concrete_ninja = Arc::new(subjects::Ninja);

        let any_ninja: Arc<dyn Any + Send + Sync> = concrete_ninja;

        let debug_ninja_result = any_ninja.cast::<dyn Debug>();

        assert!(debug_ninja_result.is_ok());
    }
}
