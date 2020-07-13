#![cfg_attr(not(feature = "std"), no_std)]
mod handle;
mod implementations;

pub use handle::Handle;

#[derive(Debug)]
pub struct InvalidEncoding;

pub trait Canon: Sized {
    fn write(&self, sink: &mut impl Sink);
    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding>;
}

pub trait Sink {
    /// Type used as key for the sink
    type Ident;

    /// Request n bytes from the Sink to be written with the value.
    /// Can be called multiple times, each time yielding consecutive byte slices
    fn request_bytes(&mut self, num_bytes: usize) -> &mut [u8];

    /// Copy from the address into the sink
    fn provide_bytes(&mut self, bytes: &[u8]);

    /// Write the value and return the corresponding Ident
    fn fin(self) -> Self::Ident;

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
    type Ident;
    type Sink: Sink<Ident = Self::Ident>;
    type Source: Source;
    type Error: From<InvalidEncoding>;

    fn sink(&'a mut self) -> Self::Sink;
    fn source(&'a self, id: &Self::Ident) -> Option<Self::Source>;

    fn put<T: Canon>(&'a mut self, t: &mut T) -> Self::Ident {
        let mut sink = self.sink();
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
