// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::{Params, State as Blake2bState};

use crate::{Canon, CanonError, Sink, Source};

/// A 32 byte Identifier based on the Blake2b hash algorithm
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct Id([u8; 32]);

/// Builder for Ids
pub struct IdBuilder(Blake2bState);

impl Canon for Id {
    fn write(&self, sink: &mut Sink) {
        sink.copy_bytes(&self.0[..])
    }

    fn read(source: &mut Source) -> Result<Self, CanonError> {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(source.read_bytes(32));
        Ok(Id(bytes))
    }

    fn encoded_len(&self) -> usize {
        32
    }
}

impl Id {
    pub(crate) fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Default for IdBuilder {
    fn default() -> Self {
        IdBuilder(Params::new().hash_length(32).to_state())
    }
}

impl IdBuilder {
    /// Write bytes into the Id hasher
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    /// Build the Id from the accumulated bytes
    pub fn fin(mut self) -> Id {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(self.0.finalize().as_ref());
        Id(bytes)
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
