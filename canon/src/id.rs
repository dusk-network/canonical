// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::{Params, State as Blake2bState};

use crate::{Canon, IdBuilder, Ident, Sink, Source, Store};

/// A 32 byte Identifier based on the Blake2b hash algorithm
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct Id32([u8; 32]);

pub struct Id32Builder(Blake2bState);

impl<S> Canon<S> for Id32
where
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        sink.copy_bytes(&self.0[..]);
        Ok(())
    }
    /// Read the value from bytes in a `Source`
    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(source.read_bytes(32));
        Ok(Id32(bytes))
    }
    fn encoded_len(&self) -> usize {
        32
    }
}

impl Default for Id32Builder {
    fn default() -> Self {
        Id32Builder(Params::new().hash_length(32).to_state())
    }
}

impl IdBuilder<Id32> for Id32Builder {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    fn fin(mut self) -> Id32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(self.0.finalize().as_ref());
        Id32(bytes)
    }
}

impl Ident for Id32 {
    type Builder = Id32Builder;
}

impl AsRef<[u8]> for Id32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Id32 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
