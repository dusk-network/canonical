use core::marker::PhantomData;

use crate::{Canon, CanonError, Sink, Source, Store};

macro_rules! number {
    ($number:ty, $size:expr) => {
        impl<S: Store> Canon<S> for $number {
            fn write(
                &self,
                sink: &mut impl Sink<S>,
            ) -> Result<(), CanonError<S::Error>> {
                sink.copy_bytes(&self.to_be_bytes());
                Ok(())
            }

            fn read(
                source: &mut impl Source<S>,
            ) -> Result<Self, CanonError<S::Error>> {
                let mut bytes = [0u8; $size];
                bytes.copy_from_slice(source.read_bytes($size));
                Ok(<$number>::from_be_bytes(bytes))
            }

            fn encoded_len(&self) -> usize {
                $size
            }
        }
    };
}

number!(u8, 1);
number!(i8, 1);

number!(u16, 2);
number!(i16, 2);

number!(u32, 4);
number!(i32, 4);

number!(u64, 8);
number!(i64, 8);

number!(u128, 16);
number!(i128, 16);

impl<T, S, const N: usize> Canon<S> for [T; N]
where
    T: Canon<S> + Default + Copy,
    S: Store,
{
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        for i in 0..N {
            self[i].write(sink)?;
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        let mut array = [T::default(); N];
        for i in 0..N {
            array[i] = T::read(source)?;
        }
        Ok(array)
    }

    fn encoded_len(&self) -> usize {
        let mut len = 0;
        for i in 0..N {
            len += self[i].encoded_len();
        }
        len
    }
}

impl<S> Canon<S> for bool
where
    S: Store,
{
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        match self {
            true => sink.copy_bytes(&[1]),
            false => sink.copy_bytes(&[0]),
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        match source.read_bytes(1) {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err(CanonError::InvalidData),
        }
    }

    fn encoded_len(&self) -> usize {
        1
    }
}

impl<T, S> Canon<S> for Option<T>
where
    T: Canon<S>,
    S: Store,
{
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        match self {
            None => sink.copy_bytes(&[0]),
            Some(t) => {
                sink.copy_bytes(&[1]);
                t.write(sink)?;
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        match source.read_bytes(1) {
            [0] => Ok(None),
            [1] => Ok(Some(T::read(source)?)),
            _ => Err(CanonError::InvalidData),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            Some(t) => 1 + t.encoded_len(),
            None => 1,
        }
    }
}

impl<T, E, S> Canon<S> for Result<T, E>
where
    T: Canon<S>,
    E: Canon<S>,
    S: Store,
{
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        match self {
            Ok(t) => {
                sink.copy_bytes(&[0]);
                t.write(sink)
            }
            Err(e) => {
                sink.copy_bytes(&[1]);
                e.write(sink)
            }
        }
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        match source.read_bytes(1) {
            [0] => Ok(Ok(T::read(source)?)),
            [1] => Ok(Err(E::read(source)?)),
            _ => Err(CanonError::InvalidData),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            Ok(t) => 1 + t.encoded_len(),
            Err(e) => 1 + e.encoded_len(),
        }
    }
}

impl<S: Store> Canon<S> for () {
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), CanonError<S::Error>> {
        Ok(())
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

impl<S: Store, T> Canon<S> for PhantomData<T> {
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), CanonError<S::Error>> {
        Ok(())
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        Ok(PhantomData)
    }

    fn encoded_len(&self) -> usize {
        0
    }
}
