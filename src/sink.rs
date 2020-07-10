use crate::canon::Canon;

pub trait Sink {
    /// Type used as key for the sink
    type Ident;

    /// Request n bytes from the Sink to be written with the value.
    /// Can be called multiple times, each time yielding consecutive byte slices
    fn request_bytes(&mut self, num_bytes: usize) -> &mut [u8];

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

    fn fin(self) -> Self::Ident;
}
