use core::marker::PhantomData;

use crate::bridge::BridgeStore;
use crate::{Canon, CanonError, Sink, Source, Store};

/// The `Handle` type can be thought of as a host-allocating version of `Box`
#[derive(Debug, Clone)]
pub enum Handle<T, S: Store> {
    Inline {
        bytes: S::Ident,
        len: u8,
        _marker: PhantomData<T>,
    },
    Ident(S::Ident),
}

impl<T, S> Canon<S> for Handle<T, S>
where
    T: Canon<S>,
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) {
        match self {
            Handle::Inline {
                ref bytes,
                ref mut len,
                ..
            } => {
                Canon::<S>::write(&mut *len, sink);
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize])
            }
            Handle::Ident(ref ident) => {
                Canon::<S>::write(&mut 0u8, sink);
                sink.copy_bytes(&ident.as_ref());
            }
        }
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S>> {
        let len = u8::read(source)?;
        if len > 0 {
            let mut bytes = S::Ident::default();
            bytes.as_mut()[0..len as usize]
                .copy_from_slice(source.read_bytes(len as usize));
            Ok(Handle::Inline {
                bytes,
                len,
                _marker: PhantomData,
            })
        } else {
            let mut ident = S::Ident::default();
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

impl<T, S> Handle<T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// Construct a new `Handle` from value `t`
    pub fn new(mut t: T) -> Result<Self, CanonError<S>> {
        let mut buffer = S::Ident::default();
        let len = t.encoded_len();
        // can we inline the value?
        if len <= buffer.as_ref().len() {
            t.write(&mut buffer.as_mut());
            Ok(Handle::Inline {
                bytes: buffer,
                len: len as u8,
                _marker: PhantomData,
            })
        } else {
            let mut store = S::singleton();
            Ok(Handle::Ident(store.put(&mut t)?))
        }
    }
}
