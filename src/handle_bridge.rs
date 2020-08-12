use core::marker::PhantomData;

use crate::{Canon, CanonError, Sink, Source, Store};

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

impl<T, S> Canon<S> for Handle<T, S>
where
    T: Canon<S>,
    S: Store,
{
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        match self {
            Handle::Inline {
                ref bytes, ref len, ..
            } => {
                Canon::<S>::write(&*len, sink)?;
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize]);
            }
            Handle::Ident(ref ident) => {
                Canon::<S>::write(&0u8, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
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
    pub fn new(t: T) -> Self {
        // In the bridged enviroment, we assume that the put will succeed,
        // and handle any errors in the host instead

        let len = t.encoded_len();
        // can we inline the value?
        let mut buffer = S::Ident::default();
        t.write(&mut buffer.as_mut())
            .expect("Put in host should always succeed");
        if len <= buffer.as_ref().len() {
            Handle::Inline {
                bytes: buffer,
                len: len as u8,
                _marker: PhantomData,
            }
        } else {
            let store = S::singleton();
            let store_buffer = S::buffer();
            t.write(&mut &mut store_buffer[..]);
            let id = store
                .put(&store_buffer[0..len])
                .expect("Put in host should always succeed");
            Handle::Ident(id)
        }
    }

    /// Restore the value from the handle
    pub fn restore(&self) -> Result<T, CanonError<S::Error>> {
        unimplemented!()
    }
}
