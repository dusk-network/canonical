// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::marker::PhantomData;

use crate::{ByteSink, ByteSource, Canon, Sink, Source, Store};

/// The `Repr` type can be thought of as a host-allocating version of `Box`
#[derive(Debug, Clone)]
pub enum Repr<T, S: Store> {
    /// Value is stored inline in the bytes of an identifier
    Inline {
        /// Thy bytes of the ident is used for the encoded value
        bytes: S::Ident,
        /// The length of the encoded value
        len: u8,
        /// The represented type
        _marker: PhantomData<T>,
    },
    /// Value is stored host-side referenced by an identifier
    Ident(S::Ident),
}

impl<T, S> Canon<S> for Repr<T, S>
where
    T: Canon<S>,
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        match self {
            Repr::Inline {
                ref bytes, ref len, ..
            } => {
                Canon::<S>::write(&*len, sink)?;
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize]);
            }
            Repr::Ident(ref ident) => {
                Canon::<S>::write(&0u8, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let len = u8::read(source)?;
        if len > 0 {
            let mut bytes = <S as Store>::Ident::default();
            bytes.as_mut()[0..len as usize]
                .copy_from_slice(source.read_bytes(len as usize));
            Ok(Repr::Inline {
                bytes,
                len,
                _marker: PhantomData,
            })
        } else {
            let mut ident = <S as Store>::Ident::default();
            let bytes = source.read_bytes(ident.as_ref().len());
            ident.as_mut().copy_from_slice(bytes);
            Ok(Repr::Ident(ident))
        }
    }

    fn encoded_len(&self) -> usize {
        match &self {
            Repr::Inline { len, .. } => {
                // length of tag + inline value
                1 + *len as usize
            }
            Repr::Ident(id) => 1 + id.as_ref().len(),
        }
    }
}

impl<T, S> Repr<T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// Construct a new `Repr` from value `t`
    pub fn new(t: T) -> Result<Self, S::Error> {
        let store = S::singleton();

        let len = t.encoded_len();
        let mut buffer = <S as Store>::Ident::default();

        // can we inline the value?
        if len <= buffer.as_ref().len() {
            let mut sink = ByteSink::new(buffer.as_mut(), store.clone());
            t.write(&mut sink)?;

            Ok(Repr::Inline {
                bytes: buffer,
                len: len as u8,
                _marker: PhantomData,
            })
        } else {
            let id = store.put(&t)?;
            Ok(Repr::Ident(id))
        }
    }

    /// Restore the value from the repr
    pub fn restore(&self) -> Result<T, S::Error> {
        match self {
            Repr::Inline {
                bytes: ident_bytes, ..
            } => {
                let mut source =
                    ByteSource::new(ident_bytes.as_ref(), S::singleton());
                Canon::<S>::read(&mut source)
            }
            Repr::Ident(id) => {
                let store = S::singleton();
                store.get(id)
            }
        }
    }
}
