use std::collections::hash_map::{DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use parking_lot::RwLock;

use canonical::{Canon, InvalidEncoding, Sink, Source, Store};

#[derive(Default, Debug)]
struct ToyStoreInner {
    map: HashMap<[u8; 8], Vec<u8>>,
    head: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ToyStore(Arc<RwLock<ToyStoreInner>>);

struct ToySink<'a> {
    store: ToyStore,
    bytes: &'a mut [u8],
    offset: usize,
}

struct ToySource<'a> {
    store: ToyStore,
    bytes: &'a [u8],
    offset: usize,
}

impl ToyStore {
    pub fn new() -> ToyStore {
        ToyStore::default()
    }
}

impl Store for ToyStore {
    type Ident = [u8; 8];
    type Error = InvalidEncoding;

    fn put<T: Canon>(&mut self, t: &mut T) -> Result<Self::Ident, Self::Error> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = ToySink {
            store: self.clone(),
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

    fn get<T: Canon>(
        &mut self,
        id: &Self::Ident,
    ) -> Result<Option<T>, Self::Error> {
        let store = self.clone();
        self.0
            .read()
            .map
            .get(id)
            .map(|bytes| {
                let mut source = ToySource {
                    store,
                    bytes,
                    offset: 0,
                };
                T::read(&mut source)
            })
            .transpose()
    }
}

impl<'a> Sink for ToySink<'a> {
    fn write_bytes(&mut self, n: usize) -> &mut [u8] {
        let start = self.offset;
        self.offset += n;
        &mut self.bytes[start..self.offset]
    }

    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.offset;
        self.offset += bytes.len();

        println!("copy bytes {:?}", bytes);
        println!("self bytes {:?}", self.bytes);

        self.bytes[ofs..self.offset].clone_from_slice(bytes)
    }
}

impl<'a> Source for ToySource<'a> {
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }
}
