#[must_use]
pub fn create_dependency_trace(
    dependency_history: &[&'static str],
    err_dependency: &'static str,
) -> String
{
    dependency_history
        .iter()
        .map(|dep| {
            if dep == &err_dependency {
                format!("\x1b[1m{}\x1b[22m", dep)
            } else {
                (*dep).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}
