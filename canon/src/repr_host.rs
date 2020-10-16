// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use arbitrary::{self, Arbitrary};

use crate::{Canon, Sink, Source, Store};

const IDENT_TAG: u8 = 0xff;

#[derive(Clone, Debug)]
/// A Repr to a value that is either local or in storage
pub enum Repr<T, S: Store> {
    /// Value is kept reference counted locally
    Value {
        /// The reference counted value on the heap
        rc: Rc<T>,
        /// The cached identifier of the value, if any
        cached_ident: RefCell<Option<Box<S::Ident>>>,
    },
    /// Value is represented by it's Identifier
    Ident {
        /// The value identifier
        ident: S::Ident,
        /// The store where to request the value
        store: S,
    },
}

impl<T, S> Canon<S> for Repr<T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        match self {
            Repr::Value { rc, cached_ident } => {
                let len = (**rc).encoded_len();
                let ident_len = S::Ident::default().as_ref().len();

                debug_assert!(
                    ident_len <= 255,
                    "Identifier lengths > 255 is not supported at the moment"
                );

                if len <= ident_len {
                    // inline value
                    Canon::<S>::write(&mut (len as u8), sink)?;
                    Canon::<S>::write(&**rc, sink)?;
                } else {
                    Canon::<S>::write(&IDENT_TAG, sink)?;
                    let ident = sink.recur(&**rc)?;
                    *cached_ident.borrow_mut() = Some(Box::new(ident.clone()));
                    sink.copy_bytes(&ident.as_ref());
                }
            }
            Repr::Ident { ref ident, .. } => {
                Canon::<S>::write(&IDENT_TAG, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let len = u8::read(source)?;
        if len == IDENT_TAG {
            // ident tag, not a valid length
            let mut ident = S::Ident::default();
            let slice = ident.as_mut();
            let bytes = source.read_bytes(slice.len());
            slice.copy_from_slice(bytes);
            Ok(Repr::Ident {
                ident,
                store: source.store().clone(),
            })
        } else {
            // inlined value
            Ok(Repr::Value {
                rc: Rc::new(T::read(source)?),
                cached_ident: RefCell::new(None),
            })
        }
    }

    fn encoded_len(&self) -> usize {
        let ident_len = S::Ident::default().as_ref().len();

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

/// A mutable reference to a represented value
pub enum ValMut<'a, T> {
    /// Borrowed value of T
    Borrowed(&'a mut T),
}

impl<'a, T> Deref for ValMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ValMut::Borrowed(b) => b,
        }
    }
}

impl<'a, T> DerefMut for ValMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ValMut::Borrowed(b) => b,
        }
    }
}

impl<T, S> Repr<T, S>
where
    S: Store,
    T: Canon<S> + Clone,
{
    /// Construct a new `Repr` from value `t`
    pub fn new(t: T) -> Result<Self, S::Error> {
        Ok(Repr::Value {
            rc: Rc::new(t),
            cached_ident: RefCell::new(None),
        })
    }

    /// Returns the value behind the `Repr`
    pub fn restore(&self) -> Result<T, S::Error> {
        match &self {
            Repr::Value { rc, .. } => Ok((**rc).clone()),
            Repr::Ident { ident, store } => store.get(ident),
        }
    }

    /// Retrieve the value behind this representation
    pub fn val(&self) -> Result<Cow<T>, S::Error> {
        match self {
            Repr::Value { rc, .. } => Ok(Cow::Borrowed(&*rc)),
            Repr::Ident { ident, store } => {
                let t = store.get(ident)?;
                Ok(Cow::Owned(t))
            }
        }
    }

    /// Retrieve a mutable value behind this representation
    pub fn val_mut(&mut self) -> Result<ValMut<T>, S::Error> {
        match self {
            Repr::Value {
                ref mut rc,
                ref mut cached_ident,
            } => {
                // clear cache
                *cached_ident = RefCell::new(None);
                Ok(ValMut::Borrowed(Rc::make_mut(rc)))
            }
            Repr::Ident { ident, store } => {
                let _t = store.get(ident)?;
                todo!()
            }
        }
    }

    /// Unwrap or clone the contained item
    pub fn unwrap_or_clone(self) -> Result<T, S::Error> {
        match self {
            Repr::Value { rc, .. } => Ok(match Rc::try_unwrap(rc) {
                Ok(t) => t,
                Err(rc) => (*rc).clone(),
            }),
            Repr::Ident { ident, store } => store.get(&ident),
        }
    }

    /// Get the identifier for the `Repr`
    pub fn get_id(&self) -> S::Ident {
        match self {
            Repr::Value { cached_ident, rc } => {
                let mut ident_cell = cached_ident.borrow_mut();
                if let Some(ident) = &mut *ident_cell {
                    *ident.clone()
                } else {
                    let ident = S::ident(&**rc);
                    *ident_cell = Some(Box::new(ident.clone()));
                    ident
                }
            }
            Repr::Ident { ident, .. } => ident.clone(),
        }
    }
}

impl<T, S> Arbitrary for Repr<T, S>
where
    T: 'static + Canon<S> + Arbitrary,
    S: Store,
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
                let ident = S::ident(&t);
                Ok(Repr::Value {
                    rc: Rc::new(t),
                    cached_ident: RefCell::new(Some(Box::new(ident))),
                })
            }
            Kind::Ident => {
                let store = S::default();
                let ident = store.put(&t).unwrap();
                Ok(Repr::Ident { ident, store })
            }
        }
    }
}
