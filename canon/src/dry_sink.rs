use crate::{Canon, IdBuilder, Ident, Sink, Store};

/// A sink that does not write to any underlying storage
pub struct DrySink<S: Store>(<S::Ident as Ident>::Builder);

impl<S: Store> DrySink<S> {
    /// Create a new DrySink
    pub fn new() -> Self {
        DrySink(<S::Ident as Ident>::Builder::default())
    }
}

impl<S> Sink<S> for DrySink<S>
where
    S: Store,
{
    fn copy_bytes(&mut self, bytes: &[u8]) {
        self.0.write_bytes(bytes)
    }

    fn recur<T>(&self, t: &T) -> Result<S::Ident, S::Error>
    where
        T: Canon<S>,
    {
        let mut sink = DrySink::new();
        t.write(&mut sink)?;
        Ok(sink.fin())
    }

    fn fin(self) -> S::Ident {
        self.0.fin()
    }
}
