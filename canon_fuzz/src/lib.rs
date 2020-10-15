// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use arbitrary::{Arbitrary, Unstructured};
use canonical::{ByteSink, Canon, Store};

const FUZZ_ITERATIONS: usize = 1024;

fn hash<T: Hash>(t: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

/// Fuzzes a type with regards to its Canon implementation.
/// making sure every serialization produces an Equal result when deserialized
pub fn fuzz_canon<C, S>(store: S)
where
    C: Canon<S> + Arbitrary + PartialEq + std::fmt::Debug,
    S: Store,
{
    fuzz_canon_iterations::<C, S>(FUZZ_ITERATIONS, store)
}

/// Fuzzes for a set number of iterations
pub fn fuzz_canon_iterations<C, S>(iterations: usize, store: S)
where
    C: Canon<S> + Arbitrary + PartialEq + std::fmt::Debug,
    S: Store,
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

        let mut sink_a = ByteSink::new(&mut buffer_a[..], store.clone());
        let mut sink_b = ByteSink::new(&mut buffer_b[..], store.clone());

        Canon::write(&canon, &mut sink_a).unwrap();
        Canon::write(&canon, &mut sink_b).unwrap();

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
            println!("content: {:?}", &canon);
            for i in 0..claimed_len {
                if buffer_a[i] != buffer_b[i] {
                    panic!("claimed {}, wrote {}", claimed_len, i - 1)
                }
            }
        }

        let id = store.put(&canon).unwrap();
        let restored = store.get(&id).unwrap();

        assert!(canon == restored);
    }
}

pub fn canon_encoding<C, S>(canon: &C) -> usize
where
    C: Canon<S> + Arbitrary + PartialEq + std::fmt::Debug,
    S: Store,
{
    let store = S::default();
    let buffer_size = canon.encoded_len();

    let mut buffer_a = vec![0xff; buffer_size];
    let mut buffer_b = vec![0x00; buffer_size];

    let mut sink_a = ByteSink::new(&mut buffer_a, store.clone());
    let mut sink_b = ByteSink::new(&mut buffer_b, store.clone());

    Canon::write(canon, &mut sink_a).unwrap();
    Canon::write(canon, &mut sink_b).unwrap();

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
