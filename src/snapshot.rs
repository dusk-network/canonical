use core::marker::PhantomData;

use crate::canon::Canon;
use crate::store::Store;

/// A snapshot of a host-alloctated value.
pub struct Snapshot<T: ?Sized, S: Store>(S::Ident, PhantomData<T>);

/// Trait to make a snapshot from a `Canon` value.
pub trait Snap {
    /// Make a snapshot in store `S` of the value.
    fn snapshot<S: Store>(&mut self) -> Result<Snapshot<Self, S>, S::Error>;
}

impl<T> Snap for T
where
    T: Canon,
{
    fn snapshot<S: Store>(&mut self) -> Result<Snapshot<Self, S>, S::Error> {
        let id = S::put(self)?;
        Ok(Snapshot(id, PhantomData))
    }
}

impl<T, S> Snapshot<T, S>
where
    S: Store,
    T: Canon,
{
    /// Extracts the value from the snapshot
    pub fn restore(&self) -> Result<T, S::Error> {
        S::get::<T>(&self.0)
    }
}
