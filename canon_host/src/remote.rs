// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::{Canon, Sink, Source, Store};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Remote<S: Store> {
    id: S::Ident,
    store: S,
}

impl<S: Store> Remote<S> {
    pub fn new<T: Canon<S>>(from: T, store: &S) -> Result<Self, S::Error> {
        let id = store.put(&from)?;
        Ok(Remote {
            id,
            store: store.clone(),
        })
    }

    pub fn cast<T: Canon<S>>(&self) -> Result<T, S::Error> {
        self.store.get(&self.id)
    }

    pub fn cast_mut<T: Canon<S>>(&mut self) -> Result<CastMut<T, S>, S::Error> {
        let t = self.store.get(&self.id)?;
        Ok(CastMut {
            remote: self,
            value: t,
        })
    }
}

#[derive(Debug)]
pub struct CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    remote: &'a mut Remote<S>,
    value: T,
}

impl<'a, T, S> Deref for CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<'a, T, S> DerefMut for CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T, S> CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    pub fn commit(&mut self) -> Result<(), S::Error> {
        let id = self.remote.store.put(&self.value)?;
        self.remote.id = id;
        Ok(())
    }
}

impl<S: Store> Canon<S> for Remote<S> {
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(sink.copy_bytes(self.id.as_ref()))
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let mut id = S::Ident::default();
        let slice = id.as_mut();
        let len = slice.len();
        slice.copy_from_slice(source.read_bytes(len));
        Ok(Remote {
            id,
            store: source.store().clone(),
        })
    }

    fn encoded_len(&self) -> usize {
        S::Ident::default().as_ref().len()
    }
}
