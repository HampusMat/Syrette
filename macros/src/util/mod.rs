pub mod error;
pub mod item_impl;
pub mod iterator_ext;
pub mod string;
pub mod syn_path;

macro_rules! to_option {
    ($($tokens: tt)+) => {
        Some($($tokens)+)
    };

    () => {
        None
    };
}

macro_rules! or {
    (($($tokens: tt)+) else ($($default: tt)*)) => {
        $($tokens)*
    };

    (() else ($($default: tt)*)) => {
        $($default)*
    };
}

pub(crate) use {or, to_option};
