use crate::{Canon, ConstantLength, InvalidEncoding, Sink, Source};

impl Canon for u8 {
    fn write(&self, sink: &mut impl Sink) {
        sink.write_bytes(1)[0] = *self
    }

    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding> {
        Ok(source.read_bytes(1)[0])
    }
}

impl ConstantLength for u8 {
    const LEN: usize = 1;
}

impl Canon for u64 {
    fn write(&self, sink: &mut impl Sink) {
        sink.copy_bytes(&self.to_be_bytes());
    }

    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding> {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(source.read_bytes(8));
        Ok(u64::from_be_bytes(bytes))
    }
}

impl ConstantLength for u64 {
    const LEN: usize = 8;
}

impl<T, const N: usize> ConstantLength for [T; N]
where
    T: ConstantLength,
{
    const LEN: usize = N * T::LEN;
}
