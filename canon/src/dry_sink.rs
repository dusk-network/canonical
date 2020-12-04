// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Canon, IdBuilder, Ident, Sink, Store};

/// A sink that does not write to any underlying storage
pub struct DrySink<S: Store>(<S::Ident as Ident>::Builder);

impl<S: Store> DrySink<S> {
    /// Create a new DrySink
    pub fn new() -> Self {
        DrySink(<S::Ident as Ident>::Builder::default())
    }
}

impl<S: Store> Default for DrySink<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Sink<S> for DrySink<S>
where
    S: Store,
{
    fn copy_bytes(&mut self, bytes: &[u8]) {
        self.0.write_bytes(bytes)
    }

    fn recur<T>(&self, t: &T) -> Result<S::Ident, S::Error>
    where
        T: Canon<S>,
    {
        let mut sink = DrySink::new();
        t.write(&mut sink)?;
        sink.fin()
    }

    /// Consume the sink and return the ident of written data
    fn fin(self) -> Result<S::Ident, S::Error> {
        Ok(self.0.fin())
    }
}
