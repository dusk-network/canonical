// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::marker::PhantomData;

use crate::canon::{Canon, InvalidEncoding};
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

    fn recur<T: Canon<BridgeStore<I>>>(
        &mut self,
        _t: &T,
    ) -> Result<
        <BridgeStore<I> as Store>::Ident,
        <BridgeStore<I> as Store>::Error,
    > {
        let full = core::mem::replace(&mut self.bytes, &mut []);
        let (a, _b) = full.split_at_mut(self.offset);
        self.bytes = a;
        unimplemented!()
        // Ok(BridgeSink {
        //     bytes: b,
        //     offset: 0,
        // })
    }
}

struct BridgeSource<'a, I> {
    bytes: &'a [u8],
    offset: usize,
    store: BridgeStore<I>,
}

impl<'a, I> Source<BridgeStore<I>> for BridgeSource<'a, I>
where
    I: Ident,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }

    fn store(&self) -> &BridgeStore<I> {
        &self.store
    }
}

#[derive(Debug)]
pub enum BridgeStoreError {
    InvalidEncoding,
}

impl<S: Store> Canon<S> for BridgeStoreError {
    fn write(&self, _sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(())
    }

    fn read(_source: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(BridgeStoreError::InvalidEncoding)
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

impl From<InvalidEncoding> for BridgeStoreError {
    fn from(_: InvalidEncoding) -> Self {
        BridgeStoreError::InvalidEncoding
    }
}

impl<I> Store for BridgeStore<I>
where
    I: Ident,
{
    type Ident = I;
    type Error = BridgeStoreError;

    fn get<T: Canon<Self>>(&self, _id: &Self::Ident) -> Result<T, Self::Error> {
        loop {}
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, Self::Error> {
        unsafe {
            let mut ret = Self::Ident::default();
            b_put(&bytes[0], bytes.len() as u32, &mut ret.as_mut()[0]);
            Ok(ret)
        }
    }

    fn put<T: Canon<Self>>(&self, _t: &T) -> Result<Self::Ident, Self::Error> {
        // unsafe {
        //     b_put(&self.buffer[0]);
        // }
        unimplemented!()
    }

    fn singleton() -> Self {
        BridgeStore {
            buffer: [0u8; BUF_SIZE],
            _marker: PhantomData,
        }
    }
}

extern "C" {
    pub fn b_put(buffer: &u8, len: u32, ret: &mut u8);
    #[allow(unused)]
    pub fn b_get(buffer: &u8);
}
