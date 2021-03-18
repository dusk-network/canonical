// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Canon, CanonError, Sink, Source, Store};

const VERSION: u8 = 0;
const LEN_BYTES: usize = 2;

/// The size of the Id payload, used to store cryptographic hashes or inlined
/// values
pub const PAYLOAD_BYTES: usize = 32;

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
///
/// To avoid unnecessary copying, the encoding of the Id in memory, is exactly
/// equal to it's `canonical` encoding.
///
/// This allows us, for example, to decode them direcly from the memory of a
/// webassembly callsite.
#[derive(Hash, PartialEq, Eq, Default, Clone, Copy, Debug, PartialOrd, Ord)]
#[repr(C)]
pub struct Id {
    version: u8,
    len: [u8; LEN_BYTES],
    payload: [u8; PAYLOAD_BYTES],
}

impl Id {
    /// Creates a new Id from bytes
    pub fn new(bytes: &[u8]) -> Self {
        let len = bytes.len();

        let version = VERSION;

        let payload = if len > PAYLOAD_BYTES {
            // Hash data
            Store::hash(&bytes[..])
        } else {
            // Inline data
            let mut inline_bytes = [0u8; PAYLOAD_BYTES];
            inline_bytes[0..len].copy_from_slice(bytes);
            inline_bytes
        };

        Id {
            version,
            len: (len as u16).to_be_bytes(),
            payload,
        }
    }

    /// Returns the bytes of the identifier
    pub fn payload(&self) -> &[u8; PAYLOAD_BYTES] {
        &self.payload
    }

    /// Consumes the Id and returns the payload bytes
    pub fn into_payload(self) -> [u8; PAYLOAD_BYTES] {
        self.payload
    }

    /// Returns the length of the represented data
    pub fn size(&self) -> usize {
        u16::from_be_bytes(self.len) as usize
    }
}

impl Canon for Id {
    fn encode(&self, sink: &mut Sink) {
        let len = u16::from_be_bytes(self.len) as usize;

        self.version.encode(sink);

        sink.copy_bytes(&self.len);

        let payload_size = core::cmp::min(len, PAYLOAD_BYTES);

        sink.copy_bytes(&self.payload[..payload_size]);
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let version = u8::decode(source)?;
        debug_assert!(version == VERSION);

        let len = u16::decode(source)?;

        let mut payload = [0u8; PAYLOAD_BYTES];
        let len_buf = len.to_be_bytes();

        let payload_size = core::cmp::min(len as usize, PAYLOAD_BYTES);

        payload[..payload_size]
            .copy_from_slice(source.read_bytes(payload_size));

        Ok(Id {
            version,
            len: len_buf,
            payload,
        })
    }

    fn encoded_len(&self) -> usize {
        let len = u16::from_be_bytes(self.len) as usize;
        let payload_size = core::cmp::min(len, PAYLOAD_BYTES);
        1 + 2 + payload_size
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use canonical_fuzz::{fuzz_canon, Arbitrary, ArbitraryError, Unstructured};

    #[test]
    fn self_representation() {
        const N: usize = 64;
        let mut vec = vec![];
        const ID_BYTES: usize = 1 + LEN_BYTES + PAYLOAD_BYTES;

        let mut buf = [0u8; ID_BYTES];

        for o in 0..N {
            let id = Id::new(&vec);

            for u in 0..o {
                vec.push((o ^ u) as u8)
            }

            let mut sink = Sink::new(&mut buf);

            id.encode(&mut sink);

            let as_bytes: &[u8; ID_BYTES] =
                unsafe { core::mem::transmute(&id) };

            assert_eq!(&buf, as_bytes);
        }
    }

    impl Arbitrary for Id {
        fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self, ArbitraryError> {
            let bytes = Vec::arbitrary(u)?;

            Ok(Id::new(&bytes))
        }
    }

    #[test]
    fn test_empty() {
        let mut source = Source::new(&[0, 0, 0]);
        let id = Id::decode(&mut source).unwrap();
        assert_eq!(id, Id::default());
    }

    fn fuzz() {
        fuzz_canon::<Id>()
    }
}
