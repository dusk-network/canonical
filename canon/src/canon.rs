// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::store::{Sink, Source};

use alloc::vec::Vec;

/// The possible errors when fetching/decoding values from a store
#[derive(Debug, Clone)]
pub enum CanonError {
    /// The byte sequence is not a valid representation of the type decoded
    InvalidEncoding,
    /// The instance could not be found in storage
    NotFound,
}

impl Canon for CanonError {
    fn encode(&self, sink: &mut Sink) {
        let byte = match self {
            CanonError::InvalidEncoding => 0,
            CanonError::NotFound => 1,
        };
        sink.copy_bytes(&[byte])
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        match u8::decode(source)? {
            0 => Ok(CanonError::InvalidEncoding),
            1 => Ok(CanonError::NotFound),
            _ => Err(CanonError::InvalidEncoding),
        }
    }

    fn encoded_len(&self) -> usize {
        1
    }
}

/// Helper trait to encode Canon types into byte vectors.
pub trait EncodeToVec {
    /// Encode `Self` into a buffer
    fn encode_to_vec(&self) -> Vec<u8>;
}

impl<T> EncodeToVec for T
where
    T: Canon,
{
    fn encode_to_vec(&self) -> Vec<u8> {
        let len = self.encoded_len();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || 0);
        let mut sink = Sink::new(&mut vec);
        self.encode(&mut sink);
        vec
    }
}

/// Trait to read/write values as bytes
pub trait Canon: Sized + Clone {
    /// Write the value as bytes to a `Sink`
    fn encode(&self, sink: &mut Sink);
    /// Read the value from bytes in a `Source`
    fn decode(source: &mut Source) -> Result<Self, CanonError>;
    /// Returns the number of bytes needed to encode this value
    fn encoded_len(&self) -> usize;
}
