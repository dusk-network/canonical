use core::marker::PhantomData;

use crate::canon::Canon;
use crate::store::Store;

pub struct Snapshot<T: ?Sized, S: Store>(S::Ident, PhantomData<T>);

pub trait Snap {
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
    pub fn restore(&self) -> Result<T, S::Error> {
        S::get::<T>(&self.0)
    }
}
