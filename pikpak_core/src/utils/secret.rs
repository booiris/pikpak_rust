use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Password(String);

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Password(*****)")
    }
}

impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Password(*****)")
    }
}

impl From<String> for Password {
    fn from(s: String) -> Self {
        Password(s)
    }
}

impl From<&str> for Password {
    fn from(s: &str) -> Self {
        Password(s.to_string())
    }
}

impl From<Password> for String {
    fn from(p: Password) -> String {
        p.0
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
