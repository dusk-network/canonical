#![no_std]
mod implementations;

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
    type Sink: Sink;
    type Source: Source;

    fn sink(&'a mut self) -> Self::Sink;
    fn source(&'a self, id: &Self::Ident) -> Option<Self::Source>;
}
