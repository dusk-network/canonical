// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::ops::{Deref, DerefMut};

/// No-std compatible alternative to `std::Cow`
pub enum Cow<'a, T> {
    /// An owned instance of `T`
    Owned(T),
    /// A borrowed instance of `T`
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
