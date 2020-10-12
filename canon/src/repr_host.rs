// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{Canon, Sink, Source, Store};

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

impl<T, S> PartialEq for Repr<T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
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
                    // write ident tag 0xff
                    Canon::<S>::write(&0xffu8, sink)?;
                    let ident = sink.recur(&**rc)?;
                    *cached_ident.borrow_mut() = Some(Box::new(ident.clone()));
                    sink.copy_bytes(&ident.as_ref());
                }
            }
            Repr::Ident { ref ident, .. } => {
                // write ident tag 0xff
                Canon::<S>::write(&0xffu8, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let len = u8::read(source)?;
        if len == 0xff {
            // ident tag
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
                // If the length is larger nthan `ident_len` + 1,
                // The value will not be inlined, and saved as tag + ident
                1 + core::cmp::max(rc.encoded_len() as usize, ident_len)
            }
            Repr::Ident { .. } => 1 + ident_len,
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

    /// Retrieve a mutable value behind this representation and run a closure on it
    pub fn val_mut<R, F>(&mut self, f: F) -> Result<R, S::Error>
    where
        F: FnOnce(&mut T) -> Result<R, S::Error>,
    {
        match self {
            Repr::Value {
                ref mut rc,
                ref mut cached_ident,
            } => {
                // clear cache
                *cached_ident = RefCell::new(None);
                f(Rc::make_mut(rc))
            }
            Repr::Ident { ident, store } => {
                let mut t = store.get(ident)?;
                let ret = f(&mut t);
                *self = Repr::Value {
                    rc: Rc::new(t),
                    cached_ident: RefCell::new(None),
                };
                ret
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
