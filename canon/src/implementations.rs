// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![allow(clippy::empty_loop)]
use core::marker::PhantomData;
use core::mem;
use dusk_varint::VarInt;

use crate::{Canon, CanonError, Sink, Source};

impl Canon for u8 {
    fn encode(&self, sink: &mut Sink) {
        sink.copy_bytes(&self.to_be_bytes())
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let mut bytes = [0u8; 1];
        bytes.copy_from_slice(source.read_bytes(1));
        Ok(u8::from_be_bytes(bytes))
    }

    fn encoded_len(&self) -> usize {
        1
    }
}

macro_rules! varint {
    ($varint:ty) => {
        impl Canon for $varint {
            fn encode(&self, sink: &mut Sink) {
                // Varint uses 7 bits per byte to encode a value.
                // So, to encode for example a `u64`, we would need:
                // 64 / 7 = 9.14 bytes
                // ==> For `u64` we need a buffer of 10 bytes.
                const BUFSIZE: usize = mem::size_of::<$varint>() * 8 / 7 + 1;
                let mut buf = [0u8; BUFSIZE];
                let len = self.encoded_len();
                self.encode_var(&mut buf);
                sink.copy_bytes(&buf[..len]);
            }

            fn decode(source: &mut Source) -> Result<Self, $crate::CanonError> {
                const MSB: u8 = 0b1000_0000;
                let varint_len = source.bytes[source.offset..]
                    .iter()
                    .take_while(|b| *b & MSB != 0)
                    .count()
                    + 1;
                VarInt::decode_var(source.read_bytes(varint_len))
                    .map_or(Err(CanonError::InvalidEncoding), |(number, _)| {
                        Ok(number)
                    })
            }

            fn encoded_len(&self) -> usize {
                self.required_space()
            }
        }
    };
}

varint!(u16);
varint!(i16);

varint!(u32);
varint!(i32);

varint!(u64);
varint!(i64);

impl Canon for u128 {
    fn encode(&self, sink: &mut Sink) {
        let high: u64 = (self >> 64) as u64;
        let low: u64 = *self as u64;

        high.encode(sink);
        low.encode(sink);
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        let high = u64::decode(source)?;
        let low = u64::decode(source)?;

        Ok((low as u128) + ((high as u128) << 64))
    }

    fn encoded_len(&self) -> usize {
        let high: u64 = (self >> 64) as u64;
        let low: u64 = *self as u64;
        high.encoded_len() + low.encoded_len()
    }
}

#[inline]
fn zigzag_encode(from: i128) -> u128 {
    ((from << 1) ^ (from >> 127)) as u128
}

#[inline]
fn zigzag_decode(from: u128) -> i128 {
    ((from >> 1) ^ (-((from & 1) as i128)) as u128) as i128
}

impl Canon for i128 {
    fn encode(&self, sink: &mut Sink) {
        zigzag_encode(*self).encode(sink)
    }

    fn decode(source: &mut Source) -> Result<Self, CanonError> {
        Ok(zigzag_decode(u128::decode(source)?))
    }

    fn encoded_len(&self) -> usize {
        zigzag_encode(*self).encoded_len()
    }
}

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

    use alloc::collections::{BTreeMap, BTreeSet};
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
            let mut len = (self.len() as u64).encoded_len();
            for t in self.iter() {
                len += t.encoded_len()
            }
            len
        }
    }

    impl<T: Ord + Canon> Canon for BTreeSet<T> {
        fn encode(&self, sink: &mut Sink) {
            let len = self.len() as u64;
            len.encode(sink);
            self.iter().for_each(|item| item.encode(sink));
        }

        fn decode(source: &mut Source) -> Result<Self, CanonError> {
            let len = u64::decode(source)?;
            let mut set = BTreeSet::new();
            for _ in 0..len {
                set.insert(T::decode(source)?);
            }
            Ok(set)
        }

        fn encoded_len(&self) -> usize {
            let len = (self.len() as u64).encoded_len();
            self.iter().fold(len, |len, item| len + item.encoded_len())
        }
    }

    impl<K, V> Canon for BTreeMap<K, V>
    where
        K: Ord + Canon,
        V: Canon,
    {
        fn encode(&self, sink: &mut Sink) {
            let len = self.len() as u64;
            len.encode(sink);
            self.iter().for_each(|(k, v)| {
                k.encode(sink);
                v.encode(sink);
            });
        }

        fn decode(source: &mut Source) -> Result<Self, CanonError> {
            let len = u64::decode(source)?;
            let mut map = BTreeMap::new();
            for _ in 0..len {
                let key = K::decode(source)?;
                let value = V::decode(source)?;
                map.insert(key, value);
            }
            Ok(map)
        }

        fn encoded_len(&self) -> usize {
            let len = (self.len() as u64).encoded_len();
            self.iter().fold(len, |len, (k, v)| {
                len + k.encoded_len() + v.encoded_len()
            })
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
            let len = self.len() as u64;
            len.encoded_len() + self.as_bytes().len()
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
