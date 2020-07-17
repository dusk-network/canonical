use crate::store::{Sink, Source};

#[derive(Debug)]
pub enum CanonError {
    InvalidData,
    MissingValue,
}

pub trait Canon: Sized + EncodedLength {
    fn write(&self, sink: &mut impl Sink);
    fn read(store: &mut impl Source) -> Result<Self, CanonError>;
}

pub trait ConstantLength {
    const LEN: usize;
}

pub trait EncodedLength {
    fn encoded_len(&self) -> usize;
}

impl<T> EncodedLength for T
where
    T: ConstantLength,
{
    fn encoded_len(&self) -> usize {
        Self::LEN
    }
}
