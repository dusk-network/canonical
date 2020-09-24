// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::marker::PhantomData;

use crate::canon::{Canon, InvalidEncoding};
use crate::store::{ByteSink, ByteSource, Ident, Sink, Source, Store};

// We set the buffer size to 4kib for now, subject to change.
const BUF_SIZE: usize = 1024 * 4;

static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

/// Store usable across ffi-boundraries
#[derive(Clone)]
pub struct BridgeStore<I> {
    _marker: PhantomData<I>,
}

impl<I> BridgeStore<I>
where
    I: Ident,
{
    /// Create a new bridge store
    pub fn new() -> Self {
        BridgeStore {
            _marker: PhantomData,
        }
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
    type Error = InvalidEncoding;

    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error> {
        unsafe {
            let slice = id.as_ref();
            let id_len = slice.len();
            BUF[0..id_len].copy_from_slice(slice);
            b_get(&mut BUF);
            // b_get has written T into the buffer
            let mut source = ByteSource::new(&BUF[..], self.clone());
            Canon::<Self>::read(&mut source)
        }
    }

    fn put<T: Canon<Self>>(&self, t: &T) -> Result<Self::Ident, Self::Error> {
        unsafe {
            let len = t.encoded_len();
            let mut sink = ByteSink::new(&mut BUF, self.clone());
            Canon::<Self>::write(t, &mut sink)?;
            b_put(&mut BUF, len);
            // b_put writes an ident back in the BUF
            let mut id = Self::Ident::default();
            let id_len = id.as_ref().len();
            id.as_mut().copy_from_slice(&BUF[0..id_len]);
            Ok(id)
        }
    }

    fn singleton() -> Self {
        BridgeStore {
            _marker: PhantomData,
        }
    }
}

extern "C" {
    pub fn b_put(buf: &mut [u8; BUF_SIZE], len: usize);
    pub fn b_get(buf: &mut [u8; BUF_SIZE]);
}
