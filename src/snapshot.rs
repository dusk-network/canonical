use core::marker::PhantomData;

use crate::canon::Canon;
use crate::store::Store;

/// A snapshot of a host-alloctated value.
pub struct Snapshot<T: ?Sized, S: Store> {
    id: S::Ident,
    store: S,
    _marker: PhantomData<T>,
}

impl<T, S> Snapshot<T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// Extracts the value from the snapshot
    pub fn restore(&self) -> Result<T, S::Error> {
        self.store.get::<T>(&self.id)
    }
}
