// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::ops::{Deref, DerefMut};

use canonical::{Canon, Store};

/// A Store that can perist state.
pub trait Persistent: Store {
    fn set_root(&mut self, root: Self::Ident);
    fn get_root(&self) -> Option<Self::Ident>;
}

/// The root of the whole-network state, including
#[derive(Default)]
pub struct Root<State> {
    state: State,
}

impl<State> Root<State>
where
    State: Default,
{
    /// Creates a new root from a persistent store
    pub fn new<S>(store: S) -> Result<Self, S::Error>
    where
        S: Persistent,
        State: Canon<S>,
    {
        let state = match store.get_root() {
            Some(ref root) => store.get(root)?,
            None => Default::default(),
        };
        Ok(Root { state })
    }
}

impl<State> Deref for Root<State> {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<State> DerefMut for Root<State> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
