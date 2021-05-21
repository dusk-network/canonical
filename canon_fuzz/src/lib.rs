// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

pub use arbitrary::{Arbitrary, Error as ArbitraryError, Unstructured};
use canonical::{Canon, Id, Sink};

const FUZZ_ITERATIONS: usize = 128;

fn raw_data<'a>() -> Unstructured<'a> {
    Unstructured::new(include_bytes!("noise.bin"))
}

/// Fuzzes a type with regards to its Canon implementation.
/// making sure every serialization produces an Equal result when deserialized
pub fn fuzz_canon<'a, C>()
where
    C: Canon + Arbitrary<'a> + PartialEq + std::fmt::Debug,
{
    fuzz_canon_iterations::<C>(FUZZ_ITERATIONS)
}

/// Fuzzes for a set number of iterations
pub fn fuzz_canon_iterations<'a, C>(iterations: usize)
where
    C: Canon + Arbitrary<'a> + PartialEq + std::fmt::Debug,
{
    let data = &mut raw_data::<'a>();
    for _ in 0..iterations {
        let canon: C = Arbitrary::arbitrary(data).unwrap();

        let claimed_len = canon.encoded_len();

        let mut buffer_a = vec![];
        buffer_a.resize_with(claimed_len + 1, || 0xff);

        let mut buffer_b = vec![];
        buffer_b.resize_with(claimed_len + 1, || 0x00);

        let mut sink_a = Sink::new(&mut buffer_a[..]);
        let mut sink_b = Sink::new(&mut buffer_b[..]);

        canon.encode(&mut sink_a);
        canon.encode(&mut sink_b);

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
                    panic!(
                        "{:?}\n\nclaimed {}, wrote {}",
                        canon,
                        claimed_len,
                        i - 1
                    );
                }
            }
        }

        let id = Id::new(&canon);
        let restored = id.reify().unwrap();

        assert!(canon == restored);
    }
}
