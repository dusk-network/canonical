use lazy_static::lazy_static;
use parking_lot::RwLock;

use std::collections::HashMap;
use std::sync::Arc;

use crate::{Canon, CanonError, Id, Sink, Source};

struct InMemoryMap(HashMap<Id, Vec<u8>>);

impl InMemoryMap {
    fn new() -> Self {
        InMemoryMap(HashMap::new())
    }

    fn insert(&mut self, id: Id, bytes: Vec<u8>) {
        self.0.insert(id, bytes);
    }

    #[allow(unused)] // FIXME
    fn get(&self, id: &Id) -> Option<&[u8]> {
        self.0.get(id).map(AsRef::as_ref)
    }
}

lazy_static! {
    static ref STATIC_MAP: Arc<RwLock<InMemoryMap>> =
        Arc::new(RwLock::new(InMemoryMap::new()));
}

pub(crate) struct HostStore;

impl HostStore {
    pub(crate) fn fetch(_id: &Id, _into: &mut [u8]) -> Result<(), CanonError> {
        todo!("a");
    }

    pub(crate) fn put<T: Canon>(t: &T) -> Id {
        let len = t.encoded_len();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || 0);
        let mut sink = Sink::new(&mut vec[..]);
        t.write(&mut sink);
        let id = sink.fin();
        STATIC_MAP.write().insert(id, vec);
        id
    }

    pub(crate) fn get<T: Canon>(id: &Id) -> Result<T, CanonError> {
        match STATIC_MAP.read().get(id) {
            Some(bytes) => {
                let mut source = Source::new(bytes);
                T::read(&mut source)
            }
            None => Err(CanonError::NotFound),
        }
    }

    #[allow(unused)] // FIXME
    pub(crate) fn id<T: Canon>(t: &T) -> Id {
        // Same as put, just don't storing anything

        let len = t.encoded_len();
        let mut vec = Vec::with_capacity(len);
        vec.resize_with(len, || 0);
        let mut sink = Sink::new(&mut vec[..]);
        t.write(&mut sink);
        let id = sink.fin();
        id
    }
}
