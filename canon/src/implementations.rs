// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use core::marker::PhantomData;

use crate::{Canon, InvalidEncoding, Sink, Source, Store};

macro_rules! number {
    ($number:ty, $size:expr) => {
        impl<S: Store> Canon<S> for $number {
            fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
                sink.copy_bytes(&self.to_be_bytes());
                Ok(())
            }

            fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
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
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        for i in 0..N {
            self[i].write(sink)?;
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
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
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        match self {
            true => sink.copy_bytes(&[1]),
            false => sink.copy_bytes(&[0]),
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        match source.read_bytes(1) {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err(InvalidEncoding.into()),
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
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        match self {
            None => sink.copy_bytes(&[0]),
            Some(t) => {
                sink.copy_bytes(&[1]);
                t.write(sink)?;
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        match source.read_bytes(1) {
            [0] => Ok(None),
            [1] => Ok(Some(T::read(source)?)),
            _ => Err(InvalidEncoding.into()),
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
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
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

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        match source.read_bytes(1) {
            [0] => Ok(Ok(T::read(source)?)),
            [1] => Ok(Err(E::read(source)?)),
            _ => Err(InvalidEncoding.into()),
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
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(())
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

impl<S: Store, T> Canon<S> for PhantomData<T> {
    fn write(&self, _: &mut impl Sink<S>) -> Result<(), S::Error> {
        Ok(())
    }

    fn read(_: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(PhantomData)
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

macro_rules! canon_tuple_impls {
    ( $( $name:ident )+, $( $idx:tt )+ ) => {
        impl<S: Store, $($name,)+> Canon<S> for ($($name,)+) where $($name: Canon<S>,)+ {
            fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
                $(self.$idx.write(sink)?;)+

                Ok(())
            }

            fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
                Ok(($($name::read(source)?,)+))
            }

            fn encoded_len(&self) -> usize {
                let mut n = 0;

                $(n += self.$idx.encoded_len();)+
                n
            }
        }
    };
}

canon_tuple_impls! { A B, 0 1 }
canon_tuple_impls! { A B C, 0 1 2 }
canon_tuple_impls! { A B C D, 0 1 2 3 }
canon_tuple_impls! { A B C D E, 0 1 2 3 4 }
canon_tuple_impls! { A B C D E F, 0 1 2 3 4 5 }
canon_tuple_impls! { A B C D E F G, 0 1 2 3 4 5 6 }
canon_tuple_impls! { A B C D E F G H, 0 1 2 3 4 5 6 7 }
canon_tuple_impls! { A B C D E F G H I, 0 1 2 3 4 5 6 7 8 }
canon_tuple_impls! { A B C D E F G H I J, 0 1 2 3 4 5 6 7 8 9 }
canon_tuple_impls! { A B C D E F G H I J K, 0 1 2 3 4 5 6 7 8 9 10 }
canon_tuple_impls! { A B C D E F G H I J K L, 0 1 2 3 4 5 6 7 8 9 10 11 }
canon_tuple_impls! { A B C D E F G H I J K L M, 0 1 2 3 4 5 6 7 8 9 10 11 12 }
canon_tuple_impls! { A B C D E F G H I J K L M N, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 }
canon_tuple_impls! { A B C D E F G H I J K L M N O, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 }
canon_tuple_impls! { A B C D E F G H I J K L M N O P, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 }

#[cfg(feature = "host")]
mod std_impls {
    use super::*;

    impl<S: Store, T: Canon<S>> Canon<S> for std::vec::Vec<T> {
        fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
            let len = self.len() as u64;
            len.write(sink)?;
            for t in self.iter() {
                t.write(sink)?;
            }
            Ok(())
        }

        fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
            let mut vec = vec![];
            let len = u64::read(source)?;
            for _ in 0..len {
                vec.push(T::read(source)?);
            }
            Ok(vec)
        }

        fn encoded_len(&self) -> usize {
            // length of length
            let mut len = Canon::<S>::encoded_len(&0u64);
            for t in self.iter() {
                len += t.encoded_len()
            }
            len
        }
    }
}
