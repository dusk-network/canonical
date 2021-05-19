// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use cfg_if::cfg_if;

use core::fmt;

use crate::id::{Id, IdHash};
use crate::CanonError;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod bridge;
        use bridge::BridgeStore as Inner;
    } else {
        mod host;
        use host::HostStore as Inner;
    }
}

/// Low-level intefrace to the store logic.
pub struct Store;

impl Store {
    /// Write the byte slice into the store and return its hash
    pub fn put(bytes: &[u8]) -> IdHash {
        Inner::put(bytes)
    }

    /// Get data with the corresponding hash and write it to a buffer
    ///
    /// Note that the buffer must be of the right length to accept the data
    pub fn get(hash: &IdHash, write_to: &mut [u8]) -> Result<(), CanonError> {
        Inner::get(hash, write_to)
    }

    /// Hash a slice of bytes
    pub fn hash(bytes: &[u8]) -> IdHash {
        Inner::hash(bytes)
    }

    pub(crate) fn take_bytes(id: &Id) -> Result<Vec<u8>, CanonError> {
        Inner::take_bytes(id)
    }
}

/// Struct used in `Canon::encode` to read bytes from a buffer
pub struct Sink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a> fmt::Debug for Sink<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sink {:?}", &self.bytes[0..self.offset])
    }
}

impl<'a> Sink<'a> {
    /// Creates a new sink reading from bytes
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Sink { bytes, offset: 0 }
    }

    /// Copies bytes into the sink
    pub fn copy_bytes(&mut self, bytes: &[u8]) {
        let len = bytes.len();
        self.bytes[self.offset..self.offset + len].copy_from_slice(bytes);
        self.offset += len;
    }
}

/// Struct used in `Canon::decode` to read bytes from a buffer
pub struct Source<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Source<'a> {
    /// Creates a new source reading from bytes
    pub fn new(bytes: &'a [u8]) -> Self {
        Source { bytes, offset: 0 }
    }

    /// Reads the next n bytes from the source
    pub fn read_bytes(&mut self, n: usize) -> &[u8] {
        let old_offset = self.offset;
        self.offset += n;
        &self.bytes[old_offset..old_offset + n]
    }
}
