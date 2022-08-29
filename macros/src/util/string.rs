use once_cell::sync::Lazy;
use regex::Regex;

static CAMELCASE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-z])([A-Z])").unwrap());

pub fn camelcase_to_snakecase(camelcased: &str) -> String
{
    CAMELCASE_RE
        .replace(camelcased, "${1}_$2")
        .to_string()
        .to_lowercase()
}
