use crate::canon::Canon;

pub trait Sink {
    /// Type used as key for the sink
    type Ident;

    /// Copy from the address into the sink
    fn copy_bytes(&mut self, bytes: &[u8]);

    #[allow(unused)]
    fn recur<T>(&self, t: &T) -> Self::Ident
    where
        T: Canon,
    {
        unimplemented!()
    }

    #[allow(unused)]
    fn replace_self<T>(&mut self, with: T)
    where
        T: Canon,
    {
    }

    fn fin(self) -> Result<Self::Ident, Self::Error>;
}
