// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use arbitrary::{Arbitrary, Unstructured};
use canonical::{ByteSink, Canon, Store};
use canonical_host::MemStore;

const FUZZ_ITERATIONS: usize = 1024;

fn hash<T: Hash>(t: T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

/// Fuzzes a type with regards to its Canon implementation.
/// making sure every serialization produces an Equal result when deserialized
pub fn fuzz_canon<C, S>()
where
    C: Canon<MemStore> + Arbitrary + PartialEq + std::fmt::Debug,
    S: Store,
{
    fuzz_canon_iterations::<C, S>(FUZZ_ITERATIONS)
}

/// Fuzzes for a set number of iterations
pub fn fuzz_canon_iterations<
    C: Canon<MemStore> + Arbitrary + PartialEq + std::fmt::Debug,
    S: Store,
>(
    iterations: usize,
) {
    let store = MemStore::new();
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

        let mut buffer_a = [0xff; 1024];
        let mut buffer_b = [0x00; 1024];

        let claimed_len = canon.encoded_len();

        let mut sink_a = ByteSink::new(&mut buffer_a, store.clone());
        let mut sink_b = ByteSink::new(&mut buffer_b, store.clone());

        Canon::write(&canon, &mut sink_a).unwrap();
        Canon::write(&canon, &mut sink_b).unwrap();

        // assert we did not write past our claimed len
        assert_eq!(buffer_a[claimed_len], 0xff);
        assert_eq!(buffer_b[claimed_len], 0x00);

        // assert we did write the last byte
        if claimed_len > 0 {
            assert_eq!(buffer_a[claimed_len - 1], buffer_b[claimed_len - 1]);
        }

        let id = store.put(&canon).unwrap();
        let restored = store.get(&id).unwrap();

        assert!(canon == restored);
    }
}
