use crate::{Canon, InvalidEncoding, Sink, Source};

impl Canon for u8 {
    fn write(&self, sink: &mut impl Sink) {
        sink.request_bytes(1)[0] = *self
    }

    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding> {
        Ok(source.request_bytes(1)[0])
    }
}

impl Canon for u64 {
    fn write(&self, sink: &mut impl Sink) {
        sink.provide_bytes(&self.to_be_bytes());
    }

    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding> {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(source.request_bytes(8));
        Ok(u64::from_be_bytes(bytes))
    }
}
