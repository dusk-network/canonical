// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, Sink, Source};

#[test]
fn test_u8() {
    let mut buf = [0u8; 1];

    for i in 0..u8::MAX {
        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, u8::decode(&mut source).unwrap());
    }
}

#[test]
fn test_u16() {
    let mut buf = [0u8; 3];

    for i in 0..u16::MAX {
        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, u16::decode(&mut source).unwrap());
    }
}

#[test]
fn test_u32() {
    let mut buf = [0u8; 5];

    let mut i = 0;
    const STRIDE: u32 = u32::MAX / 1024;

    while i < u32::MAX {
        i = i.saturating_add(STRIDE);

        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, u32::decode(&mut source).unwrap());
    }
}

#[test]
fn test_u64() {
    let mut buf = [0u8; 10];

    let mut i = 0;
    const STRIDE: u64 = u64::MAX / 1024;

    while i < u64::MAX {
        i = i.saturating_add(STRIDE);

        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, u64::decode(&mut source).unwrap());
    }
}

#[test]
fn test_u128() {
    let mut buf = [0u8; 20];

    let mut i = 0;
    const STRIDE: u128 = u128::MAX / 1024;

    while i < u128::MAX {
        i = i.saturating_add(STRIDE);

        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, u128::decode(&mut source).unwrap());
    }
}

#[test]
fn test_i64() {
    let mut buf = [0u8; 10];

    let mut i = i64::MIN;
    const STRIDE: i64 = i64::MAX / 1024;

    while i < i64::MAX {
        i = i.saturating_add(STRIDE);

        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, i64::decode(&mut source).unwrap());
    }
}

#[test]
fn test_i128() {
    let mut buf = [0u8; 20];

    let mut i = i128::MIN;
    const STRIDE: i128 = i128::MAX / 1024;

    while i < i128::MAX {
        i = i.saturating_add(STRIDE);

        let mut sink = Sink::new(&mut buf);
        i.encode(&mut sink);
        drop(sink);
        let mut source = Source::new(&buf);
        assert_eq!(i, i128::decode(&mut source).unwrap());
    }
}
