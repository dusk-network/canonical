#![no_std]
use core::marker::PhantomData;
use core::panic::PanicInfo;

use canonical::{Canon, CanonError, Ident, Sink, Source, Store};

const BUF_SIZE: usize = 1024 * 4;
static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

pub struct Bridge<I> {
    _marker: PhantomData<I>,
}

struct BridgeSink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

impl<'a> Sink for BridgeSink<'a> {
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
}

struct BridgeSource<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Source for BridgeSource<'a> {
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }
}

impl<I> Store for Bridge<I>
where
    I: Ident,
{
    type Ident = I;
    type Error = CanonError;

    fn put<T: Canon>(t: &mut T) -> Result<Self::Ident, Self::Error> {
        unsafe {
            let mut ret = Self::Ident::default();
            let mut sink = BridgeSink {
                bytes: &mut BUF,
                offset: 0,
            };
            t.write(&mut sink);
            b_put(&BUF[0], t.encoded_len() as u32, &mut ret.as_mut()[0]);
            Ok(ret)
        }
    }

    fn get<T: Canon>(id: &Self::Ident) -> Result<T, Self::Error> {
        loop {}
    }
}

extern "C" {
    pub fn b_put(buffer: &u8, len: u32, ret: &mut u8);
    pub fn b_get(buffer: &mut u8);
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
