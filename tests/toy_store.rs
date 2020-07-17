use std::collections::hash_map::{DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::RwLock;

use canonical::{Canon, CanonError, Sink, Source, Store};

#[derive(Default, Debug)]
struct ToyStoreInner {
    map: HashMap<[u8; 8], Vec<u8>>,
    head: usize,
}

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref STATE: Arc<RwLock<ToyStoreInner>> = Default::default();
}

#[derive(Default, Debug, Clone)]
pub struct ToyStore;

struct ToySink<'a> {
    bytes: &'a mut [u8],
    offset: usize,
}

struct ToySource<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Store for ToyStore {
    type Ident = [u8; 8];
    type Error = CanonError;

    fn put<T: Canon>(t: &mut T) -> Result<Self::Ident, Self::Error> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = ToySink {
            bytes: &mut bytes[..],
            offset: 0,
        };

        t.write(&mut sink);

        let mut hasher = DefaultHasher::new();
        bytes[..].hash(&mut hasher);

        let hash = hasher.finish().to_be_bytes();

        STATE.write().map.insert(hash, bytes);

        Ok(hash)
    }

    fn get<T: Canon>(id: &Self::Ident) -> Result<T, Self::Error> {
        STATE
            .read()
            .map
            .get(id)
            .map(|bytes| {
                let mut source = ToySource { bytes, offset: 0 };
                T::read(&mut source)
            })
            .unwrap_or_else(|| Err(CanonError::MissingValue))
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
