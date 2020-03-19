use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum ValidationErrors {
    Email,
    Password,
}

pub struct Validator;

impl Validator {
    pub fn email(input: &str) -> Result<(), ValidationErrors> {
        let email_regex = Regex::new(r###"^.+@.+\..+$"###).expect("Couldn't create email regex");

        match email_regex.is_match(input) {
            true => Ok(()),
            false => Err(ValidationErrors::Email),
        }
    }

    pub fn password(input: &str) -> Result<(), ValidationErrors> {
        let password_regex = Regex::new(r###"^.{8,64}$"###).expect("Couldn't create password regex");
        match password_regex.is_match(input) {
            true => Ok(()),
            false => Err(ValidationErrors::Password),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validator() {
        let emails = vec![
            "dimashur@gmail.com",
            "dimashur.edu@gmail.com",
            "dima.shur.edu@gmail.com",
            "di@gmail",
            "@",
            "",
        ];
        let expected_results = vec![
            Ok(()),
            Ok(()),
            Ok(()),
            Err(ValidationErrors::Email),
            Err(ValidationErrors::Email),
            Err(ValidationErrors::Email),
        ];

        for (index, email) in emails.iter().enumerate() {
            assert_eq!(Validator::email(email), expected_results[index]);
        }
    }

    #[test]
    fn test_password_validator() {
        let passwords = vec![
            "111222444",
            "djaodjaosdj",
            "qwerty11",
            "44009922",
            "",
            "qwe",
            "123",
            "some very long password which should be rejected. the maximum allowed length is 64 characters long",
        ];
        let expected_results = vec![
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(()),
            Err(ValidationErrors::Password),
            Err(ValidationErrors::Password),
            Err(ValidationErrors::Password),
            Err(ValidationErrors::Password),
        ];

        for (index, password) in passwords.iter().enumerate() {
            assert_eq!(Validator::password(password), expected_results[index]);
        }
    }
}
