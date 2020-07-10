use crate::canon::Canon;

pub trait Source {
    /// Type used as key for the sink
    type Ident;

    /// Request n bytes from the Source to be read into your value.
    /// Can be called multiple times, each time yielding consecutive byte slices
    fn request_bytes(&mut self, num_bytes: usize) -> &[u8];
}
