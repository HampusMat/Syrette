use crate::libs::intertrait::{CasterError, GetCasterError};

#[derive(thiserror::Error, Debug)]
pub enum CastError
{
    #[error("Failed to get caster")]
    GetCasterFailed(#[from] GetCasterError),

    #[error("Failed to cast from {from} to {to}")]
    CastFailed
    {
        #[source]
        source: CasterError,
        from: &'static str,
        to: &'static str,
    },

    #[error("'{0}' can't be cast to an Arc")]
    NotArcCastable(&'static str),
}
