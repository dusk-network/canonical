// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![allow(clippy::empty_loop)]
use core::marker::PhantomData;

use crate::{Canon, CanonError, Sink, Source};

macro_rules! number {
    ($number:ty, $size:expr) => {
        impl Canon for $number {
            fn encode(&self, sink: &mut Sink) {
                sink.copy_bytes(&self.to_be_bytes())
            }

            fn decode(source: &mut Source) -> Result<Self, $crate::CanonError> {
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

impl Canon for bool {
    fn encode(&self, sink: &mut Sink) {
        match self {
            true => sink.copy_bytes(&[1]),
            false => sink.copy_bytes(&[0]),
        }
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        match source.read_bytes(1) {
            [0] => Ok(false),
            [1] => Ok(true),
            _ => Err(CanonError::InvalidEncoding),
        }
    }

    fn encoded_len(&self) -> usize {
        1
    }
}

impl<T> Canon for Option<T>
where
    T: Canon,
{
    fn encode(&self, sink: &mut Sink) {
        match self {
            None => sink.copy_bytes(&[0]),
            Some(t) => {
                sink.copy_bytes(&[1]);
                t.encode(sink);
            }
        }
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        match source.read_bytes(1) {
            [0] => Ok(None),
            [1] => Ok(Some(T::decode(source)?)),
            _ => Err(CanonError::InvalidEncoding),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            Some(t) => 1 + t.encoded_len(),
            None => 1,
        }
    }
}

impl<T, E> Canon for Result<T, E>
where
    T: Canon,
    E: Canon,
{
    fn encode(&self, sink: &mut Sink) {
        match self {
            Ok(t) => {
                sink.copy_bytes(&[0]);
                t.encode(sink)
            }
            Err(e) => {
                sink.copy_bytes(&[1]);
                e.encode(sink)
            }
        }
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        match source.read_bytes(1) {
            [0] => Ok(Ok(T::decode(source)?)),
            [1] => Ok(Err(E::decode(source)?)),
            _ => Err(CanonError::InvalidEncoding),
        }
    }

    fn encoded_len(&self) -> usize {
        match self {
            Ok(t) => 1 + t.encoded_len(),
            Err(e) => 1 + e.encoded_len(),
        }
    }
}

impl Canon for () {
    fn encode(&self, _: &mut Sink) {}

    fn decode(_: &mut Source) -> Result<Self, CanonError> {
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

impl Canon for ! {
    fn encode(&self, _: &mut Sink) {}

    fn decode(_: &mut Source) -> Result<Self, CanonError> {
        loop {}
    }

    fn encoded_len(&self) -> usize {
        loop {}
    }
}

impl<T> Canon for PhantomData<T> {
    fn encode(&self, _: &mut Sink) {}

    fn decode(_: &mut Source) -> Result<Self, CanonError> {
        Ok(PhantomData)
    }

    fn encoded_len(&self) -> usize {
        0
    }
}

macro_rules! tuple {
    ( $($name:ident)+) => (
        #[allow(non_snake_case)]
        impl<$($name,)+> Canon for ($($name,)+) where $($name: Canon,)+ {
            fn encode(&self, sink: &mut Sink) {
                let ($(ref $name,)+) = *self;
                $($name.encode(sink);)+
            }

            fn decode(source: &mut Source) -> Result<Self, CanonError> {
                Ok(($($name::decode(source)?,)+))
            }

            fn encoded_len(&self) -> usize {
                let ($(ref $name,)+) = *self;
                0 $(+ $name.encoded_len())*
            }

        }
    );
}

tuple! { A B }
tuple! { A B C }
tuple! { A B C D }
tuple! { A B C D E }
tuple! { A B C D E F }
tuple! { A B C D E F G }
tuple! { A B C D E F G H }
tuple! { A B C D E F G H I }
tuple! { A B C D E F G H I J }
tuple! { A B C D E F G H I J K }
tuple! { A B C D E F G H I J K L }
tuple! { A B C D E F G H I J K L M }
tuple! { A B C D E F G H I J K L M N }
tuple! { A B C D E F G H I J K L M N O }
tuple! { A B C D E F G H I J K L M N O P }

impl<T, const N: usize> Canon for [T; N]
where
    T: Canon + Sized,
{
    fn encode(&self, sink: &mut Sink) {
        self.iter().for_each(|item| item.encode(sink));
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        array_init::try_array_init(|_| T::decode(source))
    }

    fn encoded_len(&self) -> usize {
        self.iter().fold(0, |len, item| len + item.encoded_len())
    }
}

mod alloc_impls {
    use super::*;

    extern crate alloc;

    use alloc::rc::Rc;
    use alloc::string::String;
    use alloc::sync::Arc;
    use alloc::vec::Vec;

    impl<T: Canon> Canon for Vec<T> {
        fn encode(&self, sink: &mut Sink) {
            let len = self.len() as u64;
            len.encode(sink);
            for t in self.iter() {
                t.encode(sink);
            }
        }

        fn decode(source: &mut Source) -> Result<Self, CanonError> {
            let mut vec = Vec::new();
            let len = u64::decode(source)?;
            for _ in 0..len {
                vec.push(T::decode(source)?);
            }
            Ok(vec)
        }

        fn encoded_len(&self) -> usize {
            // length of length
            let mut len = Canon::encoded_len(&0u64);
            for t in self.iter() {
                len += t.encoded_len()
            }
            len
        }
    }

    impl Canon for String {
        fn encode(&self, sink: &mut Sink) {
            let bytes = self.as_bytes();
            let len = bytes.len() as u64;
            len.encode(sink);
            sink.copy_bytes(bytes);
        }

        fn decode(source: &mut Source) -> Result<Self, CanonError> {
            let len = u64::decode(source)?;
            let vec: Vec<u8> = source.read_bytes(len as usize).into();
            String::from_utf8(vec).map_err(|_| CanonError::InvalidEncoding)
        }

        fn encoded_len(&self) -> usize {
            Canon::encoded_len(&0u64) + self.as_bytes().len()
        }
    }

    impl<T> Canon for Rc<T>
    where
        T: Canon,
    {
        fn encode(&self, sink: &mut Sink) {
            (**self).encode(sink)
        }

        fn decode(source: &mut Source) -> Result<Self, CanonError> {
            T::decode(source).map(Rc::new)
        }

        fn encoded_len(&self) -> usize {
            (**self).encoded_len()
        }
    }

    impl<T> Canon for Arc<T>
    where
        T: Canon,
    {
        fn encode(&self, sink: &mut Sink) {
            (**self).encode(sink)
        }

        fn decode(source: &mut Source) -> Result<Self, CanonError> {
            T::decode(source).map(Arc::new)
        }

        fn encoded_len(&self) -> usize {
            (**self).encoded_len()
        }
    }
}
