#![cfg_attr(not(feature = "std"), no_std)]
mod handle;
mod implementations;

pub use handle::Handle;

pub trait ConstantLength {
    const LEN: usize;
}

pub trait EncodedLength {
    fn encoded_length(&self) -> usize;
}

impl<T> EncodedLength for T
where
    T: ConstantLength,
{
    fn encoded_length(&self) -> usize {
        Self::LEN
    }
}

#[derive(Debug)]
pub struct InvalidEncoding;

pub trait Canon: Sized + EncodedLength {
    fn write(&self, sink: &mut impl Sink);
    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding>;
}

pub trait Sink {
    /// Type used as key for the sink
    type Ident;
    type Error;

    /// Request bytes to be written.
    fn request_bytes(&mut self, n: usize) -> &mut [u8];

    /// Copy from the address into the sink
    fn provide_bytes(&mut self, bytes: &[u8]);

    /// Write the value and return the corresponding Ident
    fn fin(self) -> Result<Self::Ident, Self::Error>;

    fn recur<T>(&self, _t: &T) -> Self::Ident
    where
        T: Canon,
    {
        unimplemented!()
    }

    fn replace_self<T>(&mut self, _with: T)
    where
        T: Canon,
    {
        unimplemented!()
    }
}

pub trait Source {
    fn request_bytes(&mut self, num_bytes: usize) -> &[u8];
}

pub trait Store<'a> {
    type Ident: ConstantLength;
    type Sink: Sink<Ident = Self::Ident, Error = Self::Error>;
    type Source: Source;
    type Error: From<InvalidEncoding>;

    fn sink(&'a mut self, capacity: usize) -> Self::Sink;
    fn source(&'a self, id: &Self::Ident) -> Option<Self::Source>;

    fn put<T: Canon>(
        &'a mut self,
        t: &mut T,
    ) -> Result<Self::Ident, Self::Error> {
        let e_len = t.encoded_length();

        let mut sink = self.sink(e_len);
        t.write(&mut sink);
        sink.fin()
    }

    fn get<T: Canon>(
        &'a mut self,
        id: &Self::Ident,
    ) -> Result<Option<T>, Self::Error> {
        self.source(id)
            .map(|ref mut source| T::read(source).map_err(Into::into))
            .transpose()
    }
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<'a, S> Canon for S
where
    S: Store<'a>,
{
    fn write(&self, _sink: &mut impl Sink) {
        unimplemented!()
    }
    fn read(_source: &mut impl Source) -> Result<Self, InvalidEncoding> {
        unimplemented!()
    }
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<'a, S> ConstantLength for S
where
    S: Store<'a>,
{
    const LEN: usize = 0;
}
