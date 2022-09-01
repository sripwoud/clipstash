use crate::domain::clip::ClipError;
// use super::ClipError;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Password(Option<String>);

impl Password {
    pub fn new<T: Into<Option<String>>>(password: T) -> Result<Self, ClipError> {
        let password: Option<String> = password.into();
        match password {
            Some(password) => {
                if password.trim().is_empty() {
                    Ok(Self(Some(password)))
                } else {
                    Ok(Self(None))
                }
            }
            None => Ok(Self(None))
        }
    }

    pub fn into_inner(self) -> Option<String> {
        self.0
    }

    // need to expose this publicly to be able to check (constructor param is private)
    pub fn has_password(self) -> bool {
          self.0.is_some()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for Password {
    fn default() -> Self {
        Self(None)
    }
}

// will allow to create Passwords from borrowed strings
impl FromStr for Password {
    type Err = ClipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}