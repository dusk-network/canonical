use crate::{
    Canon, ConstantLength, EncodedLength, InvalidEncoding, Sink, Source, Store,
};
use core::marker::PhantomData;

#[derive(Debug)]
pub enum Handle<T, S: Store> {
    Inline {
        // We are re-using the ident as a byte storage and access it through
        // `AsRef<[u8]>` and `AsMut<[u8]>` from the associated trait bound.
        bytes: S::Ident,
        len: u8,
        _marker: PhantomData<T>,
    },
    Ident {
        ident: S::Ident,
        store: S,
    },
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
            _ => unimplemented!("non-inlined write"),
        }
    }

    fn read(source: &mut impl Source) -> Result<Self, InvalidEncoding> {
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
            unimplemented!("non-inlined read")
        }
    }
}

impl<T, S> Handle<T, S>
where
    T: Canon,
    S: Store,
{
    pub fn new(t: T) -> Self {
        // can we inline the value?
        let len = t.encoded_len();
        if len <= S::Ident::LEN {
            let mut buffer = S::Ident::default();
            t.write(&mut buffer.as_mut());
            Handle::Inline {
                bytes: buffer,
                len: len as u8,
                _marker: PhantomData,
            }
        } else {
            unimplemented!("aaah")
        }
    }

    pub fn resolve(&self) -> Result<T, S::Error> {
        match self {
            Handle::Inline { bytes, len, .. } => {
                T::read(&mut &bytes.as_ref()[0..*len as usize])
                    .map_err(Into::into)
            }
            Handle::Ident { .. } => unimplemented!(),
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
            _ => unimplemented!(),
        }
    }
}
