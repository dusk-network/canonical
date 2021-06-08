// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::cell::{Ref, RefCell, RefMut};
use core::ops::{Deref, DerefMut};

use alloc::rc::Rc;

use crate::id;
use crate::{Canon, CanonError, Id, Sink, Source};

#[derive(Debug)]
enum ReprInner<T> {
    Id(Id),
    IdValue(Id, Rc<T>),
    Value(Rc<T>),
    // Used for moving ReprInner out of the RefCell
    Placeholder,
}

impl<T> Default for ReprInner<T> {
    fn default() -> Self {
        ReprInner::Placeholder
    }
}

impl<T> Clone for ReprInner<T> {
    fn clone(&self) -> Self {
        match self {
            ReprInner::Id(id) => ReprInner::Id(*id),
            ReprInner::IdValue(id, val) => ReprInner::IdValue(*id, val.clone()),
            ReprInner::Value(val) => ReprInner::Value(val.clone()),
            _ => unreachable!(),
        }
    }
}

#[deprecated(
    since = "0.6.4",
    note = "Please use the Link in `microkelvin` instead"
)]
#[derive(Debug)]
/// A Repr to a value that is either local or in storage behind an identifier
pub struct Repr<T>(RefCell<ReprInner<T>>);

impl<T> Clone for Repr<T> {
    fn clone(&self) -> Self {
        Repr(self.0.clone())
    }
}

impl<T> Canon for Repr<T>
where
    T: Canon,
{
    fn encode(&self, sink: &mut Sink) {
        // move out of refcell
        let mut borrow = self.0.borrow_mut();
        // Assure that the Repr has its Id loaded
        let result = match core::mem::take(&mut *borrow) {
            ReprInner::Id(id) => {
                id.encode(sink);
                ReprInner::Id(id)
            }
            ReprInner::IdValue(id, rc) => {
                id.encode(sink);
                ReprInner::IdValue(id, rc)
            }
            ReprInner::Value(rc) => {
                let t: &T = &*rc;
                let id = Id::new(t);
                id.encode(sink);
                ReprInner::IdValue(id, rc)
            }
            ReprInner::Placeholder => unreachable!(),
        };
        *borrow = result;
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        Ok(Repr(RefCell::new(ReprInner::Id(Id::decode(source)?))))
    }

    fn encoded_len(&self) -> usize {
        // The Repr always has the same length as the Id representing the value
        match &*self.0.borrow() {
            ReprInner::Id(id) | ReprInner::IdValue(id, _) => id.encoded_len(),
            ReprInner::Value(rc) => {
                let enc_len = (*rc).encoded_len();
                let payload_len =
                    core::cmp::min(enc_len as usize, id::PAYLOAD_BYTES);
                1 + (enc_len as u32).encoded_len() + payload_len
            }
            ReprInner::Placeholder => unreachable!(),
        }
    }
}

#[derive(Debug)]
/// A reference to a value behind a `Repr`
pub struct Val<'a, T>(Ref<'a, ReprInner<T>>);

impl<'a, T> Deref for Val<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &*self.0 {
            ReprInner::Value(rc) | ReprInner::IdValue(_, rc) => &*rc,
            _ => unreachable!("Invalid typestate"),
        }
    }
}

#[derive(Debug)]
/// A mutable reference to a value behind a `Repr`
pub struct ValMut<'a, T>(RefMut<'a, ReprInner<T>>);

impl<'a, T> Deref for ValMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match &*self.0 {
            ReprInner::Value(rc) => &*rc,
            _ => panic!("Broken typestate guarantee"),
        }
    }
}

impl<'a, T> DerefMut for ValMut<'a, T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut T {
        match &mut *self.0 {
            ReprInner::Value(ref mut rc) => Rc::make_mut(rc),
            _ => panic!("Broken typestate guarantee"),
        }
    }
}

impl<T> Repr<T> {
    /// Construct a new `Repr` from value `t`
    pub fn new(t: T) -> Self {
        Repr(RefCell::new(ReprInner::Value(Rc::new(t))))
    }

    /// Retrieve the value behind this representation
    pub fn val(&self) -> Result<Val<T>, CanonError>
    where
        T: Canon,
    {
        // move out of refcell
        let mut borrow = self.0.borrow_mut();
        // Assure that the Repr has its value loaded into memory
        let result = match core::mem::take(&mut *borrow) {
            loaded @ ReprInner::Value(_)
            | loaded @ ReprInner::IdValue(_, _) => loaded,
            ReprInner::Id(id) => ReprInner::IdValue(id, Rc::new(id.reify()?)),
            ReprInner::Placeholder => unreachable!(),
        };
        *borrow = result;
        // drop mutable borrow
        drop(borrow);
        // re-borrow immutable
        Ok(Val(self.0.borrow()))
    }

    /// Retrieve a mutable value behind this representation
    pub fn val_mut(&mut self) -> Result<ValMut<T>, CanonError>
    where
        T: Canon,
    {
        // move out of refcell
        let mut borrow = self.0.borrow_mut();
        // Assures that the Repr has its value loaded into memory, is mutable,
        // and have no cached `Id`
        let result = match core::mem::take(&mut *borrow) {
            ReprInner::Value(rc) | ReprInner::IdValue(_, rc) => {
                ReprInner::Value(rc)
            }
            ReprInner::Id(id) => ReprInner::Value(Rc::new(id.reify()?)),
            ReprInner::Placeholder => unreachable!(),
        };
        *borrow = result;
        Ok(ValMut(borrow))
    }
}
