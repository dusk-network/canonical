use crate::{Canon, CanonError, Sink, Source, Store};
use core::marker::PhantomData;

/// The `Handle` type can be thought of as a host-allocating version of `Box`
#[derive(Debug)]
pub enum Handle<T, S: Store> {
    /// Value stored inline in serialized form
    Inline {
        /// We are re-using the ident as a byte storage and access it through
        /// `AsRef<[u8]>` and `AsMut<[u8]>` from the associated trait bound.
        bytes: S::Ident,
        /// The length of the encoded value
        len: u8,
        #[doc(hidden)]
        _marker: PhantomData<T>,
    },
    /// Value is host-allocated with an identifier
    Ident(S::Ident),
}

impl<T, S> Canon for Handle<T, S>
where
    S: Store,
{
    fn write(&self, sink: &mut impl Sink) {
        match self {
            Handle::Inline { bytes, len, .. } => {
                len.write(sink);
                sink.copy_bytes(&bytes.as_ref()[0..*len as usize])
            }
            Handle::Ident(ident) => {
                0u8.write(sink);
                sink.copy_bytes(&ident.as_ref());
            }
        }
    }

    fn read(source: &mut impl Source) -> Result<Self, CanonError> {
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
        match self {
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
    T: Canon,
    S: Store,
{
    /// Construct a new `Handle` from value `t`
    pub fn new(mut t: T) -> Result<Self, S::Error> {
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
            Ok(Handle::Ident(S::put(&mut t)?))
        }
    }

    /// Returns the value behind the `Handle`
    pub fn resolve(&self) -> Result<T, S::Error> {
        match self {
            Handle::Inline { bytes, len, .. } => {
                T::read(&mut &bytes.as_ref()[0..*len as usize])
                    .map_err(Into::into)
            }
            Handle::Ident(ident) => S::get(ident),
        }
    }
}
