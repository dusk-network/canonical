// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use arbitrary::{Arbitrary, Unstructured};
use canonical::{Canon, Sink, Store};

const FUZZ_ITERATIONS: usize = 64;

fn hash<T: Hash>(t: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

/// Fuzzes a type with regards to its Canon implementation.
/// making sure every serialization produces an Equal result when deserialized
pub fn fuzz_canon<C>()
where
    C: Canon + Arbitrary + PartialEq + std::fmt::Debug,
{
    fuzz_canon_iterations::<C>(FUZZ_ITERATIONS)
}

/// Fuzzes for a set number of iterations
pub fn fuzz_canon_iterations<C>(iterations: usize)
where
    C: Canon + Arbitrary + PartialEq + std::fmt::Debug,
{
    let mut entropy = 0;
    for _ in 0..iterations {
        let mut bytes = vec![];

        let canon = {
            loop {
                match C::arbitrary(&mut Unstructured::new(&bytes)) {
                    Ok(t) => break t,
                    Err(_) => {
                        entropy += 1;

                        bytes.extend_from_slice(&hash(entropy).to_be_bytes());
                    }
                }
            }
        };

        let claimed_len = canon.encoded_len();

        let mut buffer_a = vec![];
        buffer_a.resize_with(claimed_len + 1, || 0xff);

        let mut buffer_b = vec![];
        buffer_b.resize_with(claimed_len + 1, || 0x00);

        let mut sink_a = Sink::new(&mut buffer_a[..]);
        let mut sink_b = Sink::new(&mut buffer_b[..]);

        Canon::write(&canon, &mut sink_a);
        Canon::write(&canon, &mut sink_b);

        let mut valid = true;

        // assert we did not write past our claimed len

        if buffer_a[claimed_len] != 0xff
            || buffer_b[claimed_len] != 0x00
            || (claimed_len > 0
                && buffer_a[claimed_len - 1] != buffer_b[claimed_len - 1])
        {
            valid = false
        }

        if !valid {
            for i in 0..claimed_len {
                if buffer_a[i] != buffer_b[i] {
                    panic!("claimed {}, wrote {}", claimed_len, i - 1)
                }
            }
        }

        let id = Store::put(&canon);
        let restored = Store::get(&id).unwrap();

        assert!(canon == restored);
    }
}

pub fn canon_encoding<C>(canon: &C) -> usize
where
    C: Canon + Arbitrary + PartialEq + std::fmt::Debug,
{
    let buffer_size = canon.encoded_len();

    let mut buffer_a = vec![0xff; buffer_size];
    let mut buffer_b = vec![0x00; buffer_size];

    let mut sink_a = Sink::new(&mut buffer_a);
    let mut sink_b = Sink::new(&mut buffer_b);

    Canon::write(canon, &mut sink_a);
    Canon::write(canon, &mut sink_b);

    let mut len = 0;
    for i in 0..buffer_size {
        if buffer_a[i] != buffer_b[i] {
            break;
        } else {
            len += 1;
        }
    }

    len
}
