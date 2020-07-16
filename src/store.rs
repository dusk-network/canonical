use crate::canon::{Canon, ConstantLength, InvalidEncoding};

/// Restrictions on types acting as identifiers
pub trait Ident: ConstantLength + Default + AsRef<[u8]> + AsMut<[u8]> {}

impl<T> Ident for T where T: ConstantLength + Default + AsRef<[u8]> + AsMut<[u8]>
{}

pub trait Sink {
    fn write_bytes(&mut self, n: usize) -> &mut [u8];
    fn copy_bytes(&mut self, bytes: &[u8]);
}

pub trait Source {
    fn read_bytes(&mut self, n: usize) -> &[u8];
}

pub trait Store {
    type Ident: Ident;
    type Error: From<InvalidEncoding>;

    fn put<T: Canon>(&mut self, t: &mut T) -> Result<Self::Ident, Self::Error>;
    fn get<T: Canon>(
        &mut self,
        id: &Self::Ident,
    ) -> Result<Option<T>, Self::Error>;
}

/// Hack to allow the derive macro to assume stores are `Canon`
#[doc(hidden)]
impl<S> Canon for S
where
    S: Store,
{
    fn write(&self, _: &mut impl Sink) {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }

    fn read(_: &mut impl Source) -> Result<Self, InvalidEncoding> {
        unimplemented!("Stores are not Canon, hack to aid in deriving")
    }
}

#[doc(hidden)]
impl<S> ConstantLength for S
where
    S: Store,
{
    const LEN: usize = 0;
}

impl Sink for &mut [u8] {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at_mut(n);
        *self = b;
        a
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let n = bytes.len();
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at_mut(n);
        *self = b;
        a.copy_from_slice(bytes)
    }
}

impl Source for &[u8] {
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let slice = core::mem::replace(self, &mut []);
        let (a, b) = slice.split_at(n);
        *self = b;
        a
    }
}
