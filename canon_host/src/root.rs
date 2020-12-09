// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Store};

use crate::{Apply, Execute, Query, Transaction};

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

impl<State, A, R, S, const ID: u8> Apply<State, A, R, S, ID> for Root<State>
where
    State: Apply<State, A, R, S, ID>,
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<State, A, R, ID>,
    ) -> Result<R, S::Error> {
        self.state.apply(transaction)
    }
}

impl<State, A, R, S, const ID: u8> Execute<State, A, R, S, ID> for Root<State>
where
    State: Execute<State, A, R, S, ID>,
    S: Store,
{
    fn execute(&self, query: Query<State, A, R, ID>) -> Result<R, S::Error> {
        self.state.execute(query)
    }
}
