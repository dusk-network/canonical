use crate::{
    Canon, CanonError, ConstantLength, EncodedLength, Sink, Source, Store,
};
use core::marker::PhantomData;

#[derive(Debug)]
pub enum Handle<T, S: Store> {
    // Values stored inline in serialized form
    Inline {
        // We are re-using the ident as a byte storage and access it through
        // `AsRef<[u8]>` and `AsMut<[u8]>` from the associated trait bound.
        bytes: S::Ident,
        len: u8,
        _marker: PhantomData<T>,
    },
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
            let bytes = source.read_bytes(S::Ident::LEN);
            ident.as_mut().copy_from_slice(bytes);
            Ok(Handle::Ident(ident))
        }
    }
}

impl<T, S> Handle<T, S>
where
    T: Canon,
    S: Store,
{
    pub fn new(mut t: T) -> Result<Self, S::Error> {
        // can we inline the value?
        let len = t.encoded_len();
        if len <= S::Ident::LEN {
            let mut buffer = S::Ident::default();
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

impl<T, S> EncodedLength for Handle<T, S>
where
    S: Store,
{
    fn encoded_len(&self) -> usize {
        match self {
            Handle::Inline { len, .. } => {
                // length of inline value plus 1 byte tag
                *len as usize + 1
            }
            Handle::Ident(_) => S::Ident::LEN + 1,
        }
    }
}
