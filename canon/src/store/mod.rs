// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use cfg_if::cfg_if;

use crate::{Canon, CanonError, Id, IdBuilder};

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
    #[allow(unused)] // FIXME?
    /// Write bytes associated with `Ident` to provided buffer
    pub(crate) fn fetch(id: &Id, into: &mut [u8]) -> Result<(), CanonError> {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::fetch(id, into)
            } else {
                todo!("b")
            }
        }
    }

    /// Get a value from storage, given an identifier
    pub fn get<T: Canon>(id: &Id) -> Result<T, CanonError> {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::get(id)
            } else {
                todo!("b")
            }
        }
    }

    /// Encode a value into the store
    pub fn put<T: Canon>(t: &T) -> Id {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::put(t)
            } else {
                todo!("b")
            }
        }
    }

    #[allow(unused)] // FIXME?
    /// Put raw bytes in store
    pub(crate) fn put_raw(_bytes: &[u8]) -> Id {
        todo!()
    }

    /// Get the id of a type, without storing it
    pub fn id<T: Canon>(t: &T) -> Id {
        cfg_if! {
            if #[cfg(feature = "host")] {
                host::HostStore::id(t)
            } else {
                todo!("b")
            }
        }
    }
}

/// A sink over a slice of bytes
pub struct Sink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
    builder: IdBuilder,
}

impl<'a> Sink<'a> {
    /// Creates a new sink reading from bytes
    #[allow(unused)] // FIXME
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Sink {
            bytes,
            offset: 0,
            builder: Default::default(),
        }
    }

    /// Copies bytes into the sink
    pub fn copy_bytes(&mut self, bytes: &[u8]) {
        self.builder.write_bytes(bytes);
        let len = bytes.len();
        self.bytes[self.offset..self.offset + len].copy_from_slice(bytes);
        self.offset += len;
    }

    pub(crate) fn recur<T: Canon>(&self, t: &T) -> Id {
        Store::put(t)
    }

    /// Finish up the sink and return the Id of the written data
    #[allow(unused)] // FIXME
    pub fn fin(self) -> Id {
        self.builder.fin()
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
