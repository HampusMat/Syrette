pub fn camelcase_to_snakecase(camelcased: &str) -> String
{
    let mut prev_char_was_lowercase = false;

    camelcased
        .chars()
        .fold(String::new(), |mut acc, character| {
            if character.is_lowercase() {
                prev_char_was_lowercase = true;

                acc.push(character);

                return acc;
            }

            if character.is_uppercase() && prev_char_was_lowercase {
                prev_char_was_lowercase = false;

                acc.push('_');
            }

            acc.push(character.to_ascii_lowercase());

            acc
        })
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn camelcase_to_snakecase_works()
    {
        assert_eq!(camelcase_to_snakecase("LoginHandler"), "login_handler");

        assert_eq!(camelcase_to_snakecase("Transient"), "transient");

        assert_eq!(
            camelcase_to_snakecase("SystemInfoManager"),
            "system_info_manager"
        );
    }
}
