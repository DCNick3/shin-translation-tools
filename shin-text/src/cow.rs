use std::fmt::Display;

pub enum Cow<'bump, 's> {
    Borrowed(&'s str),
    Owned(bumpalo::collections::String<'bump>),
}

impl Cow<'_, '_> {
    pub fn as_str(&self) -> &str {
        match self {
            Cow::Borrowed(s) => s,
            Cow::Owned(s) => s,
        }
    }
}

impl<'s> From<&'s str> for Cow<'_, 's> {
    fn from(s: &'s str) -> Self {
        Cow::Borrowed(s)
    }
}

impl<'bump> From<bumpalo::collections::String<'bump>> for Cow<'bump, '_> {
    fn from(s: bumpalo::collections::String<'bump>) -> Self {
        Cow::Owned(s)
    }
}

impl AsRef<str> for Cow<'_, '_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'bump, 's> Display for Cow<'bump, 's> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}
