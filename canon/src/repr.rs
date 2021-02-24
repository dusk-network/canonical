// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

#[cfg(feature = "host")]
use arbitrary::{self, Arbitrary};

use alloc::boxed::Box;
use alloc::rc::Rc;

use crate::{Canon, CanonError, Id, Sink, Source, Store};

const ID_BYTES_LEN: usize = 32;

#[derive(Clone, Debug)]
/// A Repr to a value that is either local or in storage behind an identifier
pub enum Repr<T> {
    /// Value is kept reference counted locally
    Value { val: Rc<T>, id: RefCell<Option<Id>> },

    /// Value is represented by its Identifier
    Ident(Id),
}

impl<T> Canon for Repr<T>
where
    T: Canon,
{
    fn write(&self, sink: &mut Sink) {
        match self {
            Repr::Value { val, id } => {
                let len = val.encoded_len();
                (len as u16).write(sink);

                if len <= ID_BYTES_LEN {
                    // inline value
                    val.write(sink);
                } else {
                    let ident = sink.recur(&**val);
                    *id.borrow_mut() = Some(ident);
                    ident.write(sink);
                }
            }

            Repr::Ident(id) => {
                id.write(sink);
            }
        }
    }

    fn read(source: &mut Source) -> Result<Self, CanonError> {
        let len = u16::read(source)?;

        if len >= 32 {
            // ident tag, not a valid length
            let mut ident = Id::default();
            let slice = ident.as_mut();
            let bytes = source.read_bytes(slice.len());
            slice.copy_from_slice(bytes);
            Ok(Repr::Ident { ident })
        } else {
            Ok(Repr::Value {
                rc: Rc::new(T::read(source)?),
                cached_ident: RefCell::new(None),
            })
        }
    }

    fn encoded_len(&self) -> usize {
        let ident_len = core::mem::size_of::<Id>();
        debug_assert!(ident_len == 32);

        match &self {
            Repr::Value { rc, .. } => {
                // If the encoded length is larger than `ident_len`,
                // The value will not be inlined, and saved as tag + identifier
                1 + core::cmp::min(rc.encoded_len() as usize, ident_len)
            }
            Repr::Ident { .. } => 1 + ident_len,
        }
    }
}

impl<T> Repr<T>
where
    T: Canon,
{
    /// Construct a new `Repr` from value `t`
    pub fn new(t: T) -> Self {
        Repr::Value {
            rc: Rc::new(t),
            cached_ident: RefCell::new(None),
        }
    }

    /// Returns the value behind the `Repr`
    pub fn restore(&self) -> Result<T, CanonError> {
        match &self {
            Repr::Value { rc, .. } => Ok((**rc).clone()),
            Repr::Ident { ident } => Store::get(ident),
        }
    }

    /// Retrieve the value behind this representation
    pub fn val(&self) -> Result<Val<T>, CanonError> {
        match self {
            Repr::Value { rc, .. } => Ok(Val::Borrowed(&*rc)),
            Repr::Ident { ident } => {
                let t = Store::get(ident)?;
                Ok(Val::Owned(t))
            }
        }
    }

    /// Retrieve a mutable value behind this representation
    pub fn val_mut(&mut self) -> Result<ValMut<T>, CanonError> {
        match self {
            Repr::Value {
                ref mut rc,
                ref mut cached_ident,
            } => {
                // clear cache
                *cached_ident = RefCell::new(None);
                Ok(ValMut::Borrowed(Rc::make_mut(rc)))
            }
            Repr::Ident { ident } => {
                let t = Store::get(ident)?;
                Ok(ValMut::Owned {
                    value: Some(t),
                    writeback: self,
                })
            }
        }
    }

    /// Unwrap or clone the contained item
    pub fn unwrap_or_clone(self) -> Result<T, CanonError> {
        match self {
            Repr::Value { rc, .. } => Ok(match Rc::try_unwrap(rc) {
                Ok(t) => t,
                Err(rc) => (*rc).clone(),
            }),
            Repr::Ident { ident } => Store::get(&ident),
        }
    }

    /// Get the identifier for the `Repr`
    pub fn get_id(&self) -> Id {
        match self {
            Repr::Value { cached_ident, rc } => {
                let mut ident_cell = cached_ident.borrow_mut();
                if let Some(ident) = &mut *ident_cell {
                    *ident.clone()
                } else {
                    let ident = Store::id(&**rc);
                    *ident_cell = Some(Box::new(ident));
                    ident
                }
            }
            Repr::Ident { ident, .. } => *ident,
        }
    }
}

/// A reference to a value
pub enum Val<'a, T> {
    /// An owned instance of `T`
    Owned(T),
    /// A borrowed instance of `T`
    Borrowed(&'a T),
}

impl<'a, T> Deref for Val<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self {
            Val::Borrowed(t) => t,
            Val::Owned(t) => &t,
        }
    }
}

impl<'a, T> DerefMut for Val<'a, T>
where
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        if let Val::Borrowed(t) = self {
            *self = Val::Owned(t.clone())
        }
        if let Val::Owned(ref mut t) = self {
            t
        } else {
            unreachable!("")
        }
    }
}

/// A mutable value derived from a Repr
pub enum ValMut<'a, T>
where
    T: Canon,
{
    /// An owned instance of `T`
    Owned {
        /// The owned value itself, wrapped in an option to be able to move
        /// it on drop.
        value: Option<T>,
        /// Where to write back the changed value
        writeback: &'a mut Repr<T>,
    },
    /// A borrowed instance of `T`
    Borrowed(&'a mut T),
}

impl<'a, T> Deref for ValMut<'a, T>
where
    T: Canon,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ValMut::Borrowed(b) => b,
            ValMut::Owned { value, .. } => {
                value.as_ref().expect("Always Some until dropped")
            }
        }
    }
}

impl<'a, T> DerefMut for ValMut<'a, T>
where
    T: Canon,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ValMut::Borrowed(b) => b,
            ValMut::Owned { ref mut value, .. } => {
                value.as_mut().expect("Always Some until dropped")
            }
        }
    }
}

impl<'a, T> Drop for ValMut<'a, T>
where
    T: Canon,
{
    fn drop(&mut self) {
        if let ValMut::Owned { value, writeback } = self {
            let value = value.take().expect("Always Some until drop");
            **writeback = Repr::<T>::new(value);
        }
    }
}

#[cfg(feature = "host")]
impl<T> Arbitrary for Repr<T>
where
    T: 'static + Canon + Arbitrary,
{
    fn arbitrary(
        u: &mut arbitrary::Unstructured<'_>,
    ) -> arbitrary::Result<Self> {
        #[derive(Arbitrary)]
        enum Kind {
            Value,
            ValueCached,
            Ident,
        }

        let t = T::arbitrary(u)?;

        match Kind::arbitrary(u)? {
            Kind::Value => Ok(Repr::Value {
                rc: Rc::new(t),
                cached_ident: RefCell::new(None),
            }),
            Kind::ValueCached => {
                let ident = Store::put(&t);
                Ok(Repr::Value {
                    rc: Rc::new(t),
                    cached_ident: RefCell::new(Some(Box::new(ident))),
                })
            }
            Kind::Ident => {
                let ident = Store::put(&t);
                Ok(Repr::Ident { ident })
            }
        }
    }
}
