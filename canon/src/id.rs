// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use alloc::vec::Vec;

use crate::canon::{Canon, CanonError, EncodeToVec};
use crate::store::{Sink, Source, Store};

const VERSION: u8 = 0;

/// values
pub const HASH_BYTES: usize = 32;

/// A hash identifiying some data
pub type IdHash = [u8; 32];

/// This is the Id type, that uniquely identifies slices of bytes,
/// in rust equivalent to `&[u8]`. As in the case with `&[u8]` the length is
/// also encoded in the type, making it a kind of a fat-pointer for content
/// addressed byte-slices.
///
/// The length of the corresponding byte-string is encoded in the first two
/// bytes in big endian.
///
/// If the length of the byteslice is less than or equal to 32 bytes, the bytes
/// are stored directly inline in the `bytes` field.
///
/// Proposal: The trailing bytes in an inlined value MUST be set to zero
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, PartialOrd, Ord)]
pub struct Id {
    version: u8,
    len: u32,
    hash: IdHash,
}

impl core::fmt::Debug for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Id(")?;
        for byte in self.hash {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ", {})", self.len)
    }
}

impl Id {
    /// Creates a new Id from a type
    pub fn new<T>(t: &T) -> Self
    where
        T: Canon,
    {
        let bytes = t.encode_to_vec();
        let len = bytes.len();
        let hash = Store::put(&bytes);

        Id {
            version: VERSION,
            len: (len as u32),
            hash,
        }
    }

    /// Creates a new Id from raw data
    pub fn raw(hash: [u8; 32], len: u32) -> Self {
        Id {
            version: VERSION,
            len: (len as u32),
            hash,
        }
    }

    /// Returns the computed hash of the value.
    ///
    /// Note that this is different from the payload itself in case of an
    /// inlined value, that normally does not get hashed.
    ///
    /// Useful for giving a well-distributed unique id for all `Canon` types,
    /// for use in hash maps for example.
    pub fn hash(&self) -> IdHash {
        self.hash
    }

    /// Returns the length of the represented data
    pub const fn size(&self) -> usize {
        self.len as usize
    }

    /// Attempts to reify the Id as an instance of type `T`
    pub fn reify<T>(&self) -> Result<T, CanonError>
    where
        T: Canon,
    {
        let len = self.size();

        let mut buf = Vec::new();

        buf.resize_with(len, || 0);

        Store::get(&self.hash(), &mut buf)?;
        let mut source = Source::new(&buf);

        T::decode(&mut source)
    }

    /// Takes the bytes corresponding to this id out of the underlying store.
    ///
    /// If the Id is inlined, this is a no-op and returns `Ok(None)`
    pub fn take_bytes(&self) -> Result<Option<Vec<u8>>, CanonError> {
        Ok(Some(Store::take_bytes(self)?))
    }
}

impl Canon for Id {
    fn encode(&self, sink: &mut Sink) {
        self.version.encode(sink);
        self.len.encode(sink);
        sink.copy_bytes(&self.hash());
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let version = u8::decode(source)?;

        if version != 0 {
            return Err(CanonError::InvalidEncoding);
        }

        let len = u32::decode(source)?;
        let mut hash = [0u8; HASH_BYTES];

        hash[..].copy_from_slice(source.read_bytes(HASH_BYTES));

        Ok(Id { version, len, hash })
    }

    fn encoded_len(&self) -> usize {
        1 + self.len.encoded_len() + HASH_BYTES
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impl_arbitrary {
    use super::*;
    use arbitrary::{Arbitrary, Result, Unstructured};

    impl<'a> Arbitrary<'a> for Id {
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
            let mut bytevec = Vec::arbitrary(u)?;

            // randomly extend by a hash length, to overflow inlined
            if bool::arbitrary(u)? {
                let junk = Store::hash(&bytevec[..]);
                bytevec.extend_from_slice(&junk);
            }

            Ok(Id::new(&bytevec))
        }
    }
}
