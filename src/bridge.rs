use core::marker::PhantomData;

use crate::canon::{Canon, CanonError};
use crate::store::{Ident, Sink, Source, Store};

const BUF_SIZE: usize = 1024 * 4;

#[derive(Clone)]
pub struct BridgeStore<I> {
    buffer: [u8; BUF_SIZE],
    _marker: PhantomData<I>,
}

impl<I> BridgeStore<I> {
    pub fn new() -> Self {
        BridgeStore {
            buffer: [0u8; BUF_SIZE],
            _marker: PhantomData,
        }
    }
}

struct BridgeSink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a, S: Store> Sink<S> for BridgeSink<'a> {
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

    fn recur(&self) -> Self {
        let full = core::mem::replace(self.buffer, &[]);
        let (a, b) = full.split_at_mut(self.offset);
        self.buffer = a;
        BridgeSink {
            bytes: b,
            offset: 0,
        }
    }

    fn fin(self) -> Result<S::Ident, CanonError<S>> {
        let store = S::singleton();
        store.put(self.buffer[0..self.offset])
    }
}

struct BridgeSource<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a, I> Source<BridgeStore<I>> for BridgeSource<'a> {
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

    fn put<T: Canon<Self>>(
        &self,
        t: &mut T,
    ) -> Result<Self::Ident, CanonError<Self>> {
        unsafe {
            let mut ret = Self::Ident::default();
            let mut sink = BridgeSink {
                bytes: &mut self.buffer,
                offset: 0,
            };
            t.write(&mut sink);
            b_put(
                &self.buffer[0],
                t.encoded_len() as u32,
                &mut ret.as_mut()[0],
            );
            Ok(ret)
        }
    }

    fn get<T: Canon<Self>>(
        &self,
        _id: &Self::Ident,
    ) -> Result<T, CanonError<Self>> {
        loop {}
    }

    #[cfg(feature = "bridge")]
    fn singleton() -> Self {
        BridgeStore::new()
    }
}

extern "C" {
    pub fn b_put(buffer: &u8, len: u32, ret: &mut u8);
    pub fn b_get(buffer: &mut u8);
}
