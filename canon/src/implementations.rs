// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

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

macro_rules! tuple {
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
                0 $( + self.$idx.encoded_len())*
            }
        }
    };
}

tuple! { A B, 0 1 }
tuple! { A B C, 0 1 2 }
tuple! { A B C D, 0 1 2 3 }
tuple! { A B C D E, 0 1 2 3 4 }
tuple! { A B C D E F, 0 1 2 3 4 5 }
tuple! { A B C D E F G, 0 1 2 3 4 5 6 }
tuple! { A B C D E F G H, 0 1 2 3 4 5 6 7 }
tuple! { A B C D E F G H I, 0 1 2 3 4 5 6 7 8 }
tuple! { A B C D E F G H I J, 0 1 2 3 4 5 6 7 8 9 }
tuple! { A B C D E F G H I J K, 0 1 2 3 4 5 6 7 8 9 10 }
tuple! { A B C D E F G H I J K L, 0 1 2 3 4 5 6 7 8 9 10 11 }
tuple! { A B C D E F G H I J K L M, 0 1 2 3 4 5 6 7 8 9 10 11 12 }
tuple! { A B C D E F G H I J K L M N, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 }
tuple! { A B C D E F G H I J K L M N O, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 }
tuple! { A B C D E F G H I J K L M N O P, 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 }

macro_rules! array {
    ($n:expr) => {
        impl<T, S> Canon<S> for [T; $n]
        where
            T: Canon<S> + Sized,
            S: Store,
        {
            fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
                for i in 0..$n {
                    self[i].write(sink)?;
                }
                Ok(())
            }

            fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
                let mut array = arrayvec::ArrayVec::new();

                for _ in 0..$n {
                    array.push(T::read(source)?);
                }

                Ok(array
                    .into_inner()
                    .map_err(|_| ())
                    .expect("Array not full after N pushes"))
            }

            fn encoded_len(&self) -> usize {
                let mut len = 0;
                for i in 0..$n {
                    len += self[i].encoded_len();
                }
                len
            }
        }
    };
}

array!(0);
array!(1);
array!(2);
array!(3);
array!(4);
array!(5);
array!(6);
array!(7);
array!(8);
array!(9);
array!(10);
array!(11);
array!(12);
array!(13);
array!(14);
array!(15);
array!(16);
array!(17);
array!(18);
array!(19);
array!(20);
array!(21);
array!(22);
array!(23);
array!(24);
array!(25);
array!(26);
array!(27);
array!(28);
array!(29);
array!(30);
array!(31);
array!(32);

use const_arrayvec::ArrayVec;

impl<S: Store, T: Canon<S>, const N: usize> Canon<S> for ArrayVec<T, N> {
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        let len = self.len() as u64;
        len.write(sink)?;
        for t in self.iter() {
            t.write(sink)?;
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        let mut vec: ArrayVec<T, N> = ArrayVec::new();
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

#[cfg(feature = "host")]
mod std_impls {
    use super::*;

    impl<S: Store, T: Canon<S>> Canon<S> for Vec<T> {
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

    impl<S: Store> Canon<S> for String {
        fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
            let bytes = self.as_bytes();
            let len = bytes.len() as u64;
            len.write(sink)?;
            sink.copy_bytes(bytes);
            Ok(())
        }

        fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
            let len = u64::read(source)?;
            let vec: Vec<u8> = source.read_bytes(len as usize).into();
            String::from_utf8(vec).map_err(|_| InvalidEncoding.into())
        }

        fn encoded_len(&self) -> usize {
            Canon::<S>::encoded_len(&0u64) + self.as_bytes().len()
        }
    }
}
