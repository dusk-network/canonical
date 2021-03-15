// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use cfg_if::cfg_if;

use crate::{Canon, CanonError, Id};

use alloc::vec::Vec;

cfg_if! {
    if #[cfg(feature = "host")] {
        mod host;
    } else {
        mod hosted;
    }
}

/// The singleton used to access the current store
pub struct Store;

impl Store {
    /// Fetch bytes of an Id into the specified buffer
    pub fn fetch(id: &Id, into: &mut [u8]) -> Result<(), CanonError> {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::fetch(id, into)
            } else {
                hosted::BridgeStore::fetch(id, into)
            }
        }
    }

    /// Get a value from storage, given an identifier
    pub fn get<T: Canon>(id: &Id) -> Result<T, CanonError> {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::get(id)
            } else {
                hosted::BridgeStore::get(id)
            }
        }
    }

    /// Encode a value into the store
    pub fn put<T: Canon>(t: &T) -> Id {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::put(t)
            } else {
                hosted::BridgeStore::put(t)
            }
        }
    }

    /// Get the id of a type, without storing it
    pub fn id<T: Canon>(t: &T) -> Id {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::id(t)
            } else {
                hosted::BridgeStore::id(t)
            }
        }
    }

    /// Hash a slice of bytes
    pub fn hash(bytes: &[u8]) -> [u8; 32] {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::hash(bytes)
            } else {
                hosted::BridgeStore::hash(bytes)
            }
        }
    }

    /// Hash a type implementing Canon.
    pub fn canon_hash<T: Canon>(t: &T) -> [u8; 32] {
        let len = t.encoded_len();
        let mut buf = Vec::with_capacity(len);
        buf.resize_with(len, || 0);
        let mut sink = Sink::new(&mut buf[..]);
        t.encode(&mut sink);
        Self::hash(&buf[..])
    }
}

/// A sink over a slice of bytes
pub struct Sink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
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

    pub(crate) fn recur<T: Canon>(&self, t: &T) -> Id {
        Store::put(t)
    }

    /// Finish up the sink and return the Id of the written data
    pub fn fin(self) -> Id {
        Id::new(&self.bytes[0..self.offset])
    }
}

/// A sink over a slice of bytes
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
