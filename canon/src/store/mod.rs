// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Canon, CanonError, Id, IdBuilder};

/// The singleton used to access the current store
pub struct Store;

impl Store {
    /// Write bytes associated with `Ident` to provided buffer
    pub(crate) fn fetch(_id: &Id, _into: &mut [u8]) -> Result<(), CanonError> {
        todo!()
    }

    /// Get a value from storage, given an identifier
    pub(crate) fn get<T: Canon>(_id: &Id) -> Result<T, CanonError> {
        todo!()
    }

    /// Encode a value into the store
    pub(crate) fn put<T: Canon>(_t: &T) -> Id {
        todo!()
    }

    /// Put raw bytes in store
    pub(crate) fn put_raw(_bytes: &[u8]) -> Id {
        todo!()
    }

    /// Get the id of a type, without storing it
    pub(crate) fn id<T: Canon>(_t: &T) -> Id {
        todo!()
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
    pub(crate) fn new(bytes: &'a mut [u8]) -> Self {
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

    pub(crate) fn fin(self) -> Id {
        self.builder.fin()
    }
}

/// A sink over a slice of bytes
pub struct Source<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Source<'a> {
    /// Creates a new sink reading from bytes
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Source { bytes, offset: 0 }
    }

    /// Reads the next n bytes from the source
    pub fn read_bytes(&mut self, n: usize) -> &[u8] {
        let old_offset = self.offset;
        self.offset += n;
        &self.bytes[old_offset..old_offset + n]
    }
}
