use crate::{InvalidEncoding, Sink};

pub trait Canon: Sized {
    fn write(&self, sink: &mut impl Sink);
    fn read(buf: &[u8]) -> Result<Self, InvalidEncoding>;
}

impl Canon for u8 {
    fn write(&self, sink: &mut impl Sink) {
        sink.request_bytes(1)[0] = *self
    }

    fn read(buf: &[u8]) -> Result<Self, InvalidEncoding> {
        Ok(buf[0])
    }
}

impl Canon for u64 {
    fn write(&self, sink: &mut impl Sink) {
        let bytes = sink.request_bytes(8);
        bytes[0..8].copy_from_slice(&self.to_be_bytes());
    }

    fn read(buf: &[u8]) -> Result<Self, InvalidEncoding> {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&buf[0..8]);
        Ok(u64::from_be_bytes(bytes))
    }
}
