// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::marker::PhantomData;

use crate::bridge::BridgeStore;
use crate::{Canon, CanonError, Ident, Sink, Source, Store};

/// The `Handle` type can be thought of as a host-allocating version of `Box`
#[derive(Debug, Clone)]
pub enum Handle<T, S: Store> {
    /// Value is stored inline in the bytes of an identifier
    Inline {
        /// Thy bytes of the ident is used for the encoded value
        bytes: S::Ident,
        /// The length of the encoded value
        len: u8,
        #[doc(hidden)]
        _marker: PhantomData<T>,
    },
    /// Value is stored host-side referenced by an identifier
    Ident(S::Ident),
}

impl<T, I> Canon<BridgeStore<I>> for Handle<T, BridgeStore<I>>
where
    I: Ident,
    T: Canon<BridgeStore<I>>,
{
    fn write(
        &self,
        sink: &mut impl Sink<BridgeStore<I>>,
    ) -> Result<(), CanonError<<BridgeStore<I> as Store>::Error>> {
        match self {
            Handle::Inline {
                ref bytes, ref len, ..
            } => {
                Canon::<BridgeStore<I>>::write(&*len, sink)?;
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize]);
            }
            Handle::Ident(ref ident) => {
                Canon::<BridgeStore<I>>::write(&0u8, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }
        }
        Ok(())
    }

    fn read(
        source: &mut impl Source<BridgeStore<I>>,
    ) -> Result<Self, CanonError<<BridgeStore<I> as Store>::Error>> {
        let len = u8::read(source)?;
        if len > 0 {
            let mut bytes = <BridgeStore<I> as Store>::Ident::default();
            bytes.as_mut()[0..len as usize]
                .copy_from_slice(source.read_bytes(len as usize));
            Ok(Handle::Inline {
                bytes,
                len,
                _marker: PhantomData,
            })
        } else {
            let mut ident = <BridgeStore<I> as Store>::Ident::default();
            let bytes = source.read_bytes(ident.as_ref().len());
            ident.as_mut().copy_from_slice(bytes);
            Ok(Handle::Ident(ident))
        }
    }

    fn encoded_len(&self) -> usize {
        match &self {
            Handle::Inline { len, .. } => {
                // length of tag + inline value
                1 + *len as usize
            }
            Handle::Ident(id) => 1 + id.as_ref().len(),
        }
    }
}

impl<T, I> Handle<T, BridgeStore<I>>
where
    I: Ident,
    T: Canon<BridgeStore<I>>,
{
    /// Construct a new `Handle` from value `t`
    pub fn new(t: T) -> Self {
        // In the bridged enviroment, we assume that the put will succeed,
        // and handle any errors in the host instead

        let len = t.encoded_len();
        // can we inline the value?
        let mut buffer = <BridgeStore<I> as Store>::Ident::default();
        t.write(&mut buffer.as_mut()).expect("host error");
        if len <= buffer.as_ref().len() {
            Handle::Inline {
                bytes: buffer,
                len: len as u8,
                _marker: PhantomData,
            }
        } else {
            let store = BridgeStore::new();
            let id = store.put(&t).expect("host error");
            Handle::Ident(id)
        }
    }

    /// Restore the value from the handle
    pub fn restore(
        &self,
    ) -> Result<T, CanonError<<BridgeStore<I> as Store>::Error>> {
        unimplemented!()
    }
}
