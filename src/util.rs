#[cfg(not(test))]
macro_rules! use_dependency_history {
    () => {
        use $crate::dependency_history::DependencyHistory;
    };
}

#[cfg(test)]
macro_rules! use_dependency_history {
    () => {
        use $crate::dependency_history::MockDependencyHistory as DependencyHistory;
    };
}

pub(crate) use use_dependency_history;
