// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::{Canon, CanonError, Sink, Source, Store};
use std::ops::{Deref, DerefMut};

pub struct Remote<S: Store> {
    id: S::Ident,
    store: S,
}

impl<S: Store> Remote<S> {
    pub fn new<T: Canon<S>>(from: T, store: &S) -> Result<Self, CanonError> {
        let id = store.put(&from)?;
        Ok(Remote {
            id,
            store: store.clone(),
        })
    }

    pub fn query<T: Canon<S>>(&self) -> Result<T, CanonError> {
        self.store.get(&self.id)
    }

    pub fn transact<T: Canon<S>>(
        &mut self,
    ) -> Result<Transaction<T, S>, CanonError> {
        let t = self.store.get(&self.id)?;
        Ok(Transaction {
            remote: self,
            value: t,
        })
    }
}

pub struct Transaction<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    remote: &'a mut Remote<S>,
    value: T,
}

impl<'a, T, S> Deref for Transaction<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<'a, T, S> DerefMut for Transaction<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T, S> Transaction<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    pub fn commit(&mut self) -> Result<(), CanonError> {
        let id = self.remote.store.put(&self.value)?;
        self.remote.id = id;
        Ok(())
    }
}

impl<S: Store> Canon<S> for Remote<S> {
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), CanonError> {
        Ok(sink.copy_bytes(self.id.as_ref()))
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError> {
        let mut id = S::Ident::default();
        let slice = id.as_mut();
        let len = slice.len();
        slice.copy_from_slice(source.read_bytes(len));
        Ok(Remote {
            id,
            store: source.store(),
        })
    }

    fn encoded_len(&self) -> usize {
        S::Ident::default().as_ref().len()
    }
}
