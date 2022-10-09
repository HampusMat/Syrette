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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn camelcase_to_snakecase_works()
    {
        assert_eq!(camelcase_to_snakecase("LoginHandler"), "login_handler");

        assert_eq!(camelcase_to_snakecase("Regex"), "regex");
    }
}
