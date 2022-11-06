use crate::libs::intertrait::{CasterError, GetCasterError};

#[derive(thiserror::Error, Debug)]
pub enum CastError
{
    #[error("Failed to get caster")]
    GetCasterFailed(#[from] GetCasterError),

    #[error("Failed to cast from trait {from} to trait {to}")]
    CastFailed
    {
        #[source]
        source: CasterError,
        from: &'static str,
        to: &'static str,
    },

    #[error("Trait '{0}' can't be cast to Arc")]
    NotArcCastable(&'static str),
}
