use core::ops::{Deref, DerefMut};

pub enum Cow<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<'a, T> Deref for Cow<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Cow::Borrowed(t) => t,
            Cow::Owned(t) => &t,
        }
    }
}

impl<'a, T> DerefMut for Cow<'a, T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Cow::Borrowed(t) = self {
            *self = Cow::Owned(t.clone())
        }
        if let Cow::Owned(ref mut t) = self {
            t
        } else {
            unreachable!()
        }
    }
}
