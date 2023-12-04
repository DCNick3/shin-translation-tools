use std::{fmt, ops::Deref};

use bumpalo::{
    collections::{String, Vec},
    Bump,
};

pub trait Borrow<Borrowed>
where
    Borrowed: ?Sized,
{
    fn borrow(&self) -> &Borrowed;
}

pub trait ToOwned {
    type Owned<'bump>: Borrow<Self>
    where
        Self: 'bump;

    fn to_owned<'bump>(&self, bump: &'bump Bump) -> Self::Owned<'bump>;
}

impl<'bump> Borrow<str> for String<'bump> {
    fn borrow(&self) -> &str {
        self
    }
}

impl ToOwned for str {
    type Owned<'bump> = String<'bump>;

    fn to_owned<'bump>(&self, bump: &'bump Bump) -> Self::Owned<'bump> {
        String::from_str_in(self, bump)
    }
}

impl<'bump, T> Borrow<[T]> for Vec<'bump, T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T: Clone> ToOwned for [T] {
    type Owned<'bump> = Vec<'bump, T> where T: 'bump;

    fn to_owned<'bump>(&self, bump: &'bump Bump) -> Self::Owned<'bump> {
        Vec::from_iter_in(self.iter().cloned(), bump)
    }
}

pub enum Cow<'bump, 'a, B>
where
    B: 'bump + 'a + ToOwned + ?Sized,
{
    Borrowed(&'a B),
    Owned(B::Owned<'bump>),
}

impl<'a> Cow<'a, '_, str> {
    pub fn as_str(&self) -> &str {
        match self {
            Cow::Borrowed(s) => s,
            Cow::Owned(s) => s,
        }
    }
}

impl<'a> From<&'a str> for Cow<'_, 'a, str> {
    fn from(s: &'a str) -> Self {
        Cow::Borrowed(s)
    }
}

impl<'bump> From<String<'bump>> for Cow<'bump, '_, str> {
    fn from(s: String<'bump>) -> Self {
        Cow::Owned(s)
    }
}

impl AsRef<str> for Cow<'_, '_, str> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'bump, B> Deref for Cow<'bump, '_, B>
where
    B: ToOwned + ?Sized,
    <B as ToOwned>::Owned<'bump>: Borrow<B>,
{
    type Target = B;

    fn deref(&self) -> &Self::Target {
        match *self {
            Cow::Borrowed(borrowed) => borrowed,
            Cow::Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<'bump, 'a, B> fmt::Display for Cow<'bump, 'a, B>
where
    B: fmt::Display + ToOwned + ?Sized,
    <B as ToOwned>::Owned<'bump>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Borrowed(ref b) => fmt::Display::fmt(b, f),
            Self::Owned(ref o) => fmt::Display::fmt(o, f),
        }
    }
}
