use std::collections::hash_map::{DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use parking_lot::RwLock;

use canonical::{Canon, CanonError, Sink, Source, Store};

#[derive(Default, Debug)]
struct MemStoreInner {
    map: HashMap<[u8; 8], Vec<u8>>,
    head: usize,
}

#[derive(Default, Debug, Clone)]
pub struct MemStore(Arc<RwLock<MemStoreInner>>);

impl MemStore {
    pub fn new() -> Self {
        Default::default()
    }
}

struct MemSink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

struct MemSource<'a, S> {
    bytes: &'a [u8],
    offset: usize,
    store: S,
}

impl Store for MemStore {
    type Ident = [u8; 8];
    type Error = CanonError;

    fn put<T: Canon<Self>>(
        &mut self,
        t: &mut T,
    ) -> Result<Self::Ident, Self::Error> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = MemSink {
            bytes: &mut bytes[..],
            offset: 0,
        };

        t.write(&mut sink);

        let mut hasher = DefaultHasher::new();
        bytes[..].hash(&mut hasher);

        let hash = hasher.finish().to_be_bytes();

        self.0.write().map.insert(hash, bytes);

        Ok(hash)
    }

    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error> {
        self.0
            .read()
            .map
            .get(id)
            .map(|bytes| {
                let mut source = MemSource {
                    bytes,
                    offset: 0,
                    store: self.clone(),
                };
                T::read(&mut source)
            })
            .unwrap_or_else(|| Err(CanonError::MissingValue))
    }

    fn singleton() -> Self {
        unimplemented!()
    }
}

impl<'a> Sink for MemSink<'a> {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let start = self.offset;
        self.offset += n;
        &mut self.bytes[start..self.offset]
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.offset;
        self.offset += bytes.len();
        self.bytes[ofs..self.offset].clone_from_slice(bytes)
    }
}

impl<'a, S> Source<S> for MemSource<'a, S>
where
    S: Store,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }

    fn store(&self) -> S {
        self.store.clone()
    }
}
