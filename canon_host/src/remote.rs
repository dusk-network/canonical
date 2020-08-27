// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use canonical::{Canon, CanonError, Sink, Source, Store};
use std::marker::PhantomData;

pub trait Query {
    type Args;
    type Return;

    fn query(&self, args: &Self::Args) -> Self::Return;
}

pub trait Transact {
    type Args;
    type Return;

    fn transact(&mut self, args: &Self::Args) -> Self::Return;
}

#[derive(Clone)]
pub struct Remote<S: Store> {
    id: S::Ident,
    store: S,
}

impl<S: Store> Remote<S> {
    pub fn new<C: Canon<S>>(
        from: C,
        store: &S,
    ) -> Result<Self, CanonError<S::Error>> {
        let id = store.put(&from)?;
        Ok(Remote {
            id,
            store: store.clone(),
        })
    }

    pub fn cast<T>(&self) -> Cast<T, S> {
        Cast(self, PhantomData)
    }

    pub fn cast_mut<T>(&mut self) -> CastMut<T, S> {
        CastMut(self, PhantomData)
    }
}

pub struct Cast<'a, T, S>(&'a Remote<S>, PhantomData<T>)
where
    S: Store;

pub struct CastMut<'a, T, S>(&'a mut Remote<S>, PhantomData<T>)
where
    S: Store;


impl<S: Store> Canon<S> for Remote<S> {
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        sink.copy_bytes(self.id.as_ref());
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
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
        self.id.as_ref().len()
    }
}

impl<'a, T, S> Query for Cast<'a, T, S>
where
    S: Store,
    T: Canon<S> + Query,
    T::Args: Canon<S>,
    T::Return: Canon<S>,
{
    type Args = T::Args;
    type Return = Result<T::Return, CanonError<S::Error>>;

    fn query(&self, args: &Self::Args) -> Self::Return {
        let remote = self.0;
        let slf: T = remote.store.get(&remote.id)?;
        Ok(slf.query(args))
    }
}

impl<'a, T, S> Query for CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S> + Query,
    T::Args: Canon<S>,
    T::Return: Canon<S>,
{
    type Args = T::Args;
    type Return = Result<T::Return, CanonError<S::Error>>;

    fn query(&self, args: &Self::Args) -> Self::Return {
        let remote = &self.0;
        let slf: T = remote.store.get(&remote.id)?;
        Ok(slf.query(args))
    }
}

impl<'a, T, S> Transact for CastMut<'a, T, S>
where
    S: Store,
    T: Canon<S> + Transact,
    T::Args: Canon<S>,
    T::Return: Canon<S>,
{
    type Args = T::Args;
    type Return = Result<T::Return, CanonError<S::Error>>;

    fn transact(&mut self, args: &Self::Args) -> Self::Return {
        let mut remote = &mut self.0;
        let mut slf: T = remote.store.get(&remote.id)?;
        let ret = slf.transact(args);
        let new_id = remote.store.put(&slf)?;
        remote.id = new_id;
        Ok(ret)
    }
}
