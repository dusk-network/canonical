// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Canon, CanonError, Sink, Source, Store};

/// A 32 byte + length Identifier based on the Blake2b hash algorithm
/// FFI Safe
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, Debug, PartialOrd, Ord)]
#[repr(C)]
pub struct Id {
    len: u16,
    bytes: [u8; 32],
}

impl Id {
    /// Creates a new Id from bytes
    pub fn new(bytes: &[u8]) -> Self {
        let len = bytes.len();

        if len > 32 {
            // Hash data
            let hash = Store::hash(&bytes[..]);

            Id {
                bytes: hash,
                len: len as u16,
            }
        } else {
            // Inline data
            let mut inline_bytes = [0u8; 32];
            inline_bytes[0..len].copy_from_slice(bytes);

            Id {
                bytes: inline_bytes,
                len: len as u16,
            }
        }
    }

    /// Returns the bytes of the identifier
    pub fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Consumes the Id and returns the hash bytes
    pub fn into_bytes(self) -> [u8; 32] {
        self.bytes
    }

    /// Returns the length of the represented data
    pub fn len(&self) -> usize {
        self.len as usize
    }
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
