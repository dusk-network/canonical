// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use alloc::rc::Rc;

use crate::{Canon, CanonError, Id, Sink, Source};

#[derive(Clone, Debug)]
enum ReprInner<T> {
    Id(Id),
    #[allow(unused)] // FIXME
    IdValue(Id, Rc<T>),
    Value(Rc<T>),
    // Used for moving ReprInner out of the RefCell
    Placeholder,
}

#[derive(Clone, Debug)]
/// A Repr to a value that is either local or in storage behind an identifier
pub struct Repr<T>(RefCell<ReprInner<T>>);

impl<T> Canon for Repr<T>
where
    T: Canon,
{
    fn write(&self, sink: &mut Sink) {
        let new_id = match &*self.0.borrow() {
            ReprInner::Id(id) | ReprInner::IdValue(id, _) => {
                return id.write(sink)
            }
            ReprInner::Value(rc) => sink.recur(&**rc),
            ReprInner::Placeholder => unreachable!(),
        };

        let mut borrow_mut = self.0.borrow_mut();
        if let ReprInner::Value(rc) =
            core::mem::replace(&mut *borrow_mut, ReprInner::Placeholder)
        {
            *borrow_mut = ReprInner::IdValue(new_id, rc)
        } else {
            unreachable!()
        }
    }

    fn read(source: &mut Source) -> Result<Self, CanonError> {
        Ok(Repr(RefCell::new(ReprInner::Id(Id::read(source)?))))
    }

    fn encoded_len(&self) -> usize {
        // The Repr always has the same length as the Id representing the value
        match &*self.0.borrow() {
            ReprInner::Id(id) | ReprInner::IdValue(id, _) => id.encoded_len(),
            ReprInner::Value(rc) => {
                let enc_len = (*rc).encoded_len();
                if enc_len <= 32 {
                    2 + enc_len
                } else {
                    34
                }
            }
            ReprInner::Placeholder => unreachable!(),
        }
    }
}

impl<T> Repr<T>
where
    T: Canon,
{
    /// Construct a new `Repr` from value `t`
    pub fn new(t: T) -> Self {
        Repr(RefCell::new(ReprInner::Value(Rc::new(t))))
    }

    /// Retrieve the value behind this representation
    pub fn val(&self) -> Result<Val<T>, CanonError> {
        match &*self.0.borrow() {
            ReprInner::Value(rc) | ReprInner::IdValue(_, rc) => {
                Ok(Val(rc.clone(), PhantomData))
            }
            _ => todo!("FIXME"),
        }
    }

    /// Retrieve a mutable value behind this representation
    pub fn val_mut(&mut self) -> Result<ValMut<T>, CanonError> {
        todo!("FIXME")
    }

    /// Get the identifier for the `Repr`
    pub fn get_id(&self) -> Id {
        todo!("FIXME")
    }
}

/// A value retrieved from behind a Repr
pub struct Val<'a, T>(Rc<T>, PhantomData<&'a T>);

impl<'a, T> Deref for Val<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.0
    }
}

/// A mutable value derived from a Repr
pub struct ValMut<'a, T>(&'a mut T);

impl<'a, T> Deref for ValMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T> DerefMut for ValMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}
