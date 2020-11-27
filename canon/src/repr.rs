// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// Here in feature `host` we can use std

#[cfg(feature = "host")]
use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

#[cfg(feature = "host")]
use std::rc::Rc;

#[cfg(feature = "host")]
use arbitrary::{self, Arbitrary};
use cfg_if::cfg_if;

use crate::{ByteSource, Canon, Sink, Source, Store};

#[cfg(not(feature = "host"))]
use crate::ByteSink;

const IDENT_TAG: u8 = 0xff;

#[derive(Clone, Debug)]
/// A Repr to a value that is either local or in storage
pub enum Repr<T, S: Store> {
    /// Value is kept reference counted locally
    #[cfg(feature = "host")]
    Value {
        /// The reference counted value on the heap
        rc: Rc<T>,
        /// The cached identifier of the value, if any
        cached_ident: RefCell<Option<Box<S::Ident>>>,
    },

    /// Value is stored inline as bytes
    Inline {
        /// Thy bytes of the ident is used for the encoded value
        bytes: S::Ident,
        /// The length of the encoded value
        len: u8,
        /// Type of the represented type
        _marker: PhantomData<T>,
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
            #[cfg(feature = "host")]
            Repr::Value { rc, cached_ident } => {
                let len = (**rc).encoded_len();
                let ident_len = S::Ident::default().as_ref().len();

                debug_assert!(
                    ident_len <= 255,
                    "Identifier lengths > 255 is not supported at the moment"
                );

                if len <= ident_len {
                    // inline value
                    Canon::<S>::write(&(len as u8), sink)?;
                    Canon::<S>::write(&**rc, sink)?;
                } else {
                    Canon::<S>::write(&IDENT_TAG, sink)?;
                    let ident = sink.recur(&**rc)?;
                    *cached_ident.borrow_mut() = Some(Box::new(ident));
                    sink.copy_bytes(&ident.as_ref());
                }
            }

            Repr::Ident { ref ident, .. } => {
                Canon::<S>::write(&IDENT_TAG, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }

            Repr::Inline {
                ref bytes, ref len, ..
            } => {
                Canon::<S>::write(&*len, sink)?;
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize]);
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
            cfg_if! {
                if #[cfg(feature = "host")] {
                    Ok(Repr::Value {
                        rc: Rc::new(T::read(source)?),
                        cached_ident: RefCell::new(None),
                    })
                } else {
                    let mut bytes = <S as Store>::Ident::default();
                    bytes.as_mut()[0..len as usize]
                        .copy_from_slice(source.read_bytes(len as usize));
                    Ok(Repr::Inline {
                        bytes,
                        len,
                        _marker: PhantomData,
                    })
                }
            }
        }
    }

    fn encoded_len(&self) -> usize {
        let ident_len = S::Ident::default().as_ref().len();

        match &self {
            #[cfg(feature = "host")]
            Repr::Value { rc, .. } => {
                // If the encoded length is larger than `ident_len`,
                // The value will not be inlined, and saved as tag + identifier
                1 + core::cmp::min(rc.encoded_len() as usize, ident_len)
            }
            Repr::Ident { .. } => 1 + ident_len,
            Repr::Inline { len, .. } => {
                // length of tag + inline value
                1 + *len as usize
            }
        }
    }
}

impl<T, S> Repr<T, S>
where
    S: Store,
    T: Canon<S> + Clone,
{
    /// Construct a new `Repr` from value `t`
    pub fn new(t: T) -> Self {
        cfg_if! {
            if #[cfg(feature = "host")] {
                Repr::Value {
                    rc: Rc::new(t),
                    cached_ident: RefCell::new(None),
                }
            } else {
                // The Default store is always the same in hosted environments
                let store = S::default();

                let len = t.encoded_len();
                let mut buffer = <S as Store>::Ident::default();

                // can we inline the value?
                if len <= buffer.as_ref().len() {
                    let mut sink = ByteSink::new(buffer.as_mut(), store.clone());
                    t.write(&mut sink)
                        .expect("Pre-checked buffer of sufficient length");

                    Repr::Inline {
                        bytes: buffer,
                        len: len as u8,
                        _marker: PhantomData,
                    }
                } else {
                    // Here we assume that we can put something in the host.
                    // If this actually returns an error from the BridgeStore,
                    // we panic and let the host deal with it.
                    let ident = store.put(&t).expect("BridgeStore should never fail");
                    Repr::Ident { ident, store }
                }

            }
        }
    }

    /// Returns the value behind the `Repr`
    pub fn restore(&self) -> Result<T, S::Error> {
        match &self {
            #[cfg(feature = "host")]
            Repr::Value { rc, .. } => Ok((**rc).clone()),
            Repr::Ident { ident, store } => store.get(ident),
            Repr::Inline {
                bytes: ident_bytes, ..
            } => {
                let mut source =
                    ByteSource::new(ident_bytes.as_ref(), S::default());
                Canon::<S>::read(&mut source)
            }
        }
    }

    /// Retrieve the value behind this representation
    pub fn val(&self) -> Result<Val<T>, S::Error> {
        match self {
            #[cfg(feature = "host")]
            Repr::Value { rc, .. } => Ok(Val::Borrowed(&*rc)),
            Repr::Ident { ident, store } => {
                let t = store.get(ident)?;
                Ok(Val::Owned(t))
            }
            Repr::Inline { bytes, .. } => {
                let mut source = ByteSource::new(bytes.as_ref(), S::default());
                let t = Canon::<S>::read(&mut source)?;
                Ok(Val::Owned(t))
            }
        }
    }

    /// Retrieve a mutable value behind this representation
    pub fn val_mut(&mut self) -> Result<ValMut<T, S>, S::Error> {
        match self {
            #[cfg(feature = "host")]
            Repr::Value {
                ref mut rc,
                ref mut cached_ident,
            } => {
                // clear cache
                *cached_ident = RefCell::new(None);
                Ok(ValMut::Borrowed(Rc::make_mut(rc)))
            }
            Repr::Ident { ident, store } => {
                let t = store.get(ident)?;
                Ok(ValMut::Owned {
                    value: Some(t),
                    writeback: self,
                })
            }
            Repr::Inline { bytes, .. } => {
                let mut source = ByteSource::new(bytes.as_ref(), S::default());
                let t = Canon::<S>::read(&mut source)?;
                Ok(ValMut::Owned {
                    value: Some(t),
                    writeback: self,
                })
            }
        }
    }

    /// Unwrap or clone the contained item
    pub fn unwrap_or_clone(self) -> Result<T, S::Error> {
        match self {
            #[cfg(feature = "host")]
            Repr::Value { rc, .. } => Ok(match Rc::try_unwrap(rc) {
                Ok(t) => t,
                Err(rc) => (*rc).clone(),
            }),
            Repr::Ident { ident, store } => store.get(&ident),
            Repr::Inline { bytes, .. } => {
                let mut source = ByteSource::new(bytes.as_ref(), S::default());
                let t = Canon::<S>::read(&mut source)?;
                Ok(t)
            }
        }
    }

    /// Get the identifier for the `Repr`
    pub fn get_id(&self) -> S::Ident {
        match self {
            #[cfg(feature = "host")]
            Repr::Value { cached_ident, rc } => {
                let mut ident_cell = cached_ident.borrow_mut();
                if let Some(ident) = &mut *ident_cell {
                    *ident.clone()
                } else {
                    let ident = S::ident(&**rc);
                    *ident_cell = Some(Box::new(ident));
                    ident
                }
            }
            Repr::Ident { ident, .. } => *ident,
            Repr::Inline { .. } => {
                todo!();
            }
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
            unreachable!("onkel")
        }
    }
}

/// A mutable value derived from a Repr
pub enum ValMut<'a, T, S>
where
    T: Canon<S>,
    S: Store,
{
    /// An owned instance of `T`
    Owned {
        /// The owned value itself, wrapped in an option to be able to move
        /// it on drop.
        value: Option<T>,
        /// Where to write back the changed value
        writeback: &'a mut Repr<T, S>,
    },
    /// A borrowed instance of `T`
    Borrowed(&'a mut T),
}

impl<'a, T, S> Deref for ValMut<'a, T, S>
where
    T: Canon<S>,
    S: Store,
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

impl<'a, T, S> DerefMut for ValMut<'a, T, S>
where
    T: Canon<S>,
    S: Store,
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

impl<'a, T, S> Drop for ValMut<'a, T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn drop(&mut self) {
        if let ValMut::Owned { value, writeback } = self {
            let value = value.take().expect("Always Some until drop");
            **writeback = Repr::<T, S>::new(value);
        }
    }
}

#[cfg(feature = "host")]
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
                let ident = store.put(&t).expect("could not put t");
                Ok(Repr::Ident { ident, store })
            }
        }
    }
}
