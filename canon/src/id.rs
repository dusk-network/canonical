// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use alloc::vec::Vec;

use crate::canon::{Canon, CanonError, EncodeToVec};
use crate::store::{Sink, Source, Store};

const VERSION: u8 = 0;

/// The size of the Id payload, used to store cryptographic hashes or inlined
/// values
pub const PAYLOAD_BYTES: usize = 32;

// We alias Hash and Inlined versions of Payload to be able to use them
// interchangeably but with some type documentation

/// Type alias for an arbitrary Id payload, either a hash or an inlined value
pub type Payload = [u8; PAYLOAD_BYTES];
/// Type alias for a payload that is used as a hash
pub type IdHash = Payload;
/// Type alias for a payload that is used as an inlined value
pub type Inlined = Payload;

/// This is the Id type, that uniquely identifies slices of bytes,
/// in rust equivalent to `&[u8]`. As in the case with `&[u8]` the length is
/// also encoded in the type, making it a kind of a fat-pointer for content
/// adressed byteslices.
///
/// The length of the corresponding bytestring is encoed in the first two bytes
/// in big endian.
///
/// If the length of the byteslice is less than or equal to 32 bytes, the bytes
/// are stored directly inline in the `bytes` field.
///
/// Proposal: The trailing bytes in an inlined value MUST be set to zero
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, Debug, PartialOrd, Ord)]
pub struct Id {
    version: u8,
    len: u32,
    payload: Payload,
}

impl Id {
    /// Creates a new Id from a type
    pub fn new<T>(t: &T) -> Self
    where
        T: Canon,
    {
        let len = t.encoded_len();
        let payload = if len > PAYLOAD_BYTES {
            Store::put(&t.encode_to_vec())
        } else {
            let mut stack_buf = Inlined::default();
            let mut sink = Sink::new(&mut stack_buf[..len]);
            t.encode(&mut sink);
            stack_buf
        };

        assert!(len <= u32::MAX as usize, "Payload length overflow");

        Id {
            version: VERSION,
            len: (len as u32),
            payload,
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
        let len = self.size();
        if len > PAYLOAD_BYTES {
            self.payload
        } else {
            Store::hash(&self.payload[0..len])
        }
    }

    /// Returns the bytes of the identifier
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Consumes the Id and returns the payload bytes
    pub fn into_payload(self) -> [u8; PAYLOAD_BYTES] {
        self.payload
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
        // this does not yet allocate
        let mut buf = Vec::new();

        let mut source = if len > PAYLOAD_BYTES {
            // allocation happens here
            buf.resize_with(len, || 0);

            Store::get(&self.payload, &mut buf)?;
            Source::new(&buf)
        } else {
            Source::new(&self.payload[..len])
        };

        T::decode(&mut source)
    }

    /// Takes the bytes corresponding to this id out of the underlying store.
    ///
    /// If the Id is inlined, this is a no-op and returns Ok(None)
    pub fn take_bytes(&self) -> Result<Option<Vec<u8>>, CanonError> {
        if self.size() <= PAYLOAD_BYTES {
            Ok(None)
        } else {
            Ok(Some(Store::take_bytes(self)?))
        }
    }
}

impl Canon for Id {
    fn encode(&self, sink: &mut Sink) {
        self.version.encode(sink);
        self.len.encode(sink);
        let payload_size = core::cmp::min(self.size(), PAYLOAD_BYTES);
        sink.copy_bytes(&self.payload[..payload_size]);
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let version = u8::decode(source)?;

        if version != 0 {
            return Err(CanonError::InvalidEncoding);
        }

        let len = u32::decode(source)?;
        let mut payload = [0u8; PAYLOAD_BYTES];

        let payload_size = core::cmp::min(len as usize, PAYLOAD_BYTES);

        payload[..payload_size]
            .copy_from_slice(source.read_bytes(payload_size));

        Ok(Id {
            version,
            len,
            payload,
        })
    }

    fn encoded_len(&self) -> usize {
        let payload_len = core::cmp::min(self.len as usize, PAYLOAD_BYTES);
        1 + self.len.encoded_len() + payload_len
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
