// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use blake2b_simd::{Params, State as Blake2bState};

use crate::{Canon, CanonError, Sink, Source};

/// A 32 byte Identifier based on the Blake2b hash algorithm
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct Id {
    bytes: [u8; 32],
    len: u16,
}

impl Id {
    /// The length of the data refered by the id
    pub fn new(bytes: [u8; 32], len: usize) -> Self {
        Id {
            bytes,
            len: len as u16,
        }
    }

    /// Returns the bytes of the identifier
    pub fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Returns the length of the represented data
    pub fn len(&self) -> usize {
        self.len as usize
    }
}

/// Builder for Ids
pub struct IdBuilder {
    state: Blake2bState,
    len: u16,
}

impl Canon for Id {
    fn write(&self, sink: &mut Sink) {
        self.len.write(sink);
        // if the length of the encoded data fits into 32 bytes,
        // we encode it directly.
        if self.len <= 32 {
            sink.copy_bytes(&self.bytes[0..self.len as usize]);
        }
        sink.copy_bytes(&self.bytes[..]);
    }

    fn read(source: &mut Source) -> Result<Self, CanonError> {
        let len = u16::read(source)?;
        let mut bytes = [0u8; 32];
        if len <= 32 {
            bytes.copy_from_slice(source.read_bytes(len as usize));
        } else {
            bytes.copy_from_slice(source.read_bytes(32));
        }
        Ok(Id { bytes, len })
    }

    fn encoded_len(&self) -> usize {
        if self.len <= 32 {
            2 + self.len as usize
        } else {
            34
        }
    }
}

impl Default for IdBuilder {
    fn default() -> Self {
        IdBuilder {
            state: Params::new().hash_length(32).to_state(),
            len: 0,
        }
    }
}

impl IdBuilder {
    /// Create an IdBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Write bytes into the Id hasher
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.state.update(bytes);
        self.len += bytes.len() as u16;
    }

    /// Build the Id from the accumulated bytes
    pub fn fin(mut self) -> Id {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(self.state.finalize().as_ref());
        Id {
            bytes,
            len: self.len,
        }
    }
}
