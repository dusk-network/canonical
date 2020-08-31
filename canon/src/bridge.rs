// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::marker::PhantomData;

use crate::canon::{Canon, CanonError};
use crate::store::{Ident, Sink, Source, Store};

// We set the buffer size to 4kb for now, subject to change.
const BUF_SIZE: usize = 1024 * 4;

/// Store usable across ffi-boundraries
#[derive(Clone)]
pub struct BridgeStore<I> {
    _marker: PhantomData<I>,
    buffer: [u8; BUF_SIZE],
}

impl<I> BridgeStore<I>
where
    I: Ident,
{
    /// Create a new bridge store
    pub fn new() -> Self {
        BridgeStore {
            _marker: PhantomData,
            buffer: [0u8; BUF_SIZE],
        }
    }
}

struct BridgeSink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a, I> Sink<BridgeStore<I>> for BridgeSink<'a>
where
    I: Ident,
{
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let start = self.offset;
        self.offset += n;
        &mut self.bytes[start..self.offset]
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.offset;
        self.offset += bytes.len();
        self.bytes[ofs..self.offset].clone_from_slice(bytes)
    }

    fn recur(&mut self) -> Self {
        let full = core::mem::replace(&mut self.bytes, &mut []);
        let (a, b) = full.split_at_mut(self.offset);
        self.bytes = a;
        BridgeSink {
            bytes: b,
            offset: 0,
        }
    }

    fn fin(
        self,
    ) -> Result<
        <BridgeStore<I> as Store>::Ident,
        CanonError<<BridgeStore<I> as Store>::Error>,
    > {
        let store = BridgeStore::new();
        store.put_raw(&self.bytes[0..self.offset])
    }
}

struct BridgeSource<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a, I> Source<BridgeStore<I>> for BridgeSource<'a>
where
    I: Ident,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }

    fn store(&self) -> BridgeStore<I> {
        BridgeStore::new()
    }
}

impl<I> Store for BridgeStore<I>
where
    I: Ident,
{
    type Ident = I;
    type Error = ();

    fn get<T: Canon<Self>>(
        &self,
        _id: &Self::Ident,
    ) -> Result<T, CanonError<Self::Error>> {
        loop {}
    }

    fn put_raw(
        &self,
        bytes: &[u8],
    ) -> Result<Self::Ident, CanonError<Self::Error>> {
        unsafe {
            let mut ret = Self::Ident::default();
            b_put(&bytes[0], bytes.len() as u32, &mut ret.as_mut()[0]);
            Ok(ret)
        }
    }

    fn put<T: Canon<Self>>(
        &self,
        _t: &T,
    ) -> Result<Self::Ident, CanonError<Self::Error>> {
        unsafe {
            b_get(&self.buffer[0]);
            unimplemented!()
        }
    }
}

extern "C" {
    pub fn b_put(buffer: &u8, len: u32, ret: &mut u8);
    pub fn b_get(buffer: &u8);
}
