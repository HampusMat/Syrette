pub mod error;
pub mod item_impl;
pub mod iterator_ext;
pub mod string;
pub mod syn_ext;
pub mod syn_path;
pub mod tokens;

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

/// Imports the specified item, prepending 'Mock' to the item identifier if the `test`
/// configuration option is set.
///
/// # Examples
/// ```ignore
/// use_double!(crate::dependency_history::DependencyHistory);
/// ```
/// <br>
///
/// Expands to the following when `cfg(not(test))`
/// ```ignore
/// use crate::dependency_history::DependencyHistory;
/// ```
/// <br>
///
/// Expands to the following when `cfg(test)`
/// ```ignore
/// use crate::dependency_history::MockDependencyHistory as DependencyHistory;
/// ```
macro_rules! use_double {
    ($([$($part: ident),*])? $item_path_part: ident :: $($next_part: tt)+) => {
        use_double!(
            [$($($part,)*)? $item_path_part]
            $($next_part)+
        );
    };

    ([$($part: ident),*] $item_path_part: ident) => {
        #[cfg(not(test))]
        use $($part::)* $item_path_part;

        ::paste::paste! {
            #[cfg(test)]
            use $($part::)* [<Mock $item_path_part>] as $item_path_part;
        }
    };
}

pub(crate) use {or, to_option, use_double};
