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
    /// Casts a `Box` with `Self` into a `Box` with `Dest`.
    fn cast<Dest: ?Sized + 'static>(self: Box<Self>) -> Result<Box<Dest>, CastError>;
}

/// A blanket implementation of `CastBox` for traits extending `CastFrom`.
impl<CastFromSelf: ?Sized + CastFrom> CastBox for CastFromSelf
{
    fn cast<Dest: ?Sized + 'static>(self: Box<Self>) -> Result<Box<Dest>, CastError>
    {
        let caster =
            get_caster::<Dest>((*self).type_id()).map_err(CastError::GetCasterFailed)?;

        (caster.cast_box)(self.box_any()).map_err(|err| CastError::CastFailed {
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

    use super::*;
    use crate::test_utils::subjects;

    #[test]
    fn can_cast_box()
    {
        let concrete_ninja = Box::new(subjects::Ninja);

        let abstract_ninja: Box<dyn subjects::INinja> = concrete_ninja;

        let debug_ninja_result = abstract_ninja.cast::<dyn Debug>();

        assert!(debug_ninja_result.is_ok());
    }

    #[test]
    fn cannot_cast_box_wrong()
    {
        let concrete_ninja = Box::new(subjects::Ninja);

        let abstract_ninja: Box<dyn subjects::INinja> = concrete_ninja;

        let display_ninja_result = abstract_ninja.cast::<dyn Display>();

        assert!(matches!(
            display_ninja_result,
            Err(CastError::GetCasterFailed(_))
        ));
    }

    #[test]
    fn can_cast_box_from_any()
    {
        let concrete_ninja = Box::new(subjects::Ninja);

        let any_ninja: Box<dyn Any> = concrete_ninja;

        let debug_ninja_result = any_ninja.cast::<dyn Debug>();

        assert!(debug_ninja_result.is_ok());
    }
}
