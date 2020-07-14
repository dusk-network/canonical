use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use canon::{InvalidEncoding, Sink, Source, Store};

#[derive(Default, Debug)]
pub struct ToyStore(HashMap<u64, Vec<u8>>);

#[derive(Debug)]
pub struct ToySink<'a> {
    store: &'a mut HashMap<u64, Vec<u8>>,
    bytes: Vec<u8>,
    offset: usize,
}

pub struct ToySource<'a> {
    bytes: &'a [u8],
}

impl<'a> Source for ToySource<'a> {
    fn request_bytes(&mut self, num_bytes: usize) -> &[u8] {
        let slice = &self.bytes[0..num_bytes];
        self.bytes = &self.bytes[num_bytes..];
        slice
    }
}

impl ToyStore {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> Sink for ToySink<'a> {
    type Ident = u64;
    type Error = InvalidEncoding;

    fn request_bytes(&mut self, n: usize) -> &mut [u8] {
        println!("requesting {}", n);
        let start = self.offset;
        self.offset += n;
        self.bytes.resize_with(self.offset, || 0);

        println!("{:?} {:?}", self, start);

        &mut self.bytes[start..self.offset]
    }

    fn provide_bytes(&mut self, bytes: &[u8]) {
        self.offset += bytes.len();
        self.bytes.extend_from_slice(bytes)
    }

    fn fin(self) -> Result<Self::Ident, Self::Error> {
        let mut hasher = DefaultHasher::new();
        self.bytes.hash(&mut hasher);
        let id = hasher.finish();
        self.store.insert(id, self.bytes);
        Ok(id)
    }
}

impl<'a> Store<'a> for ToyStore {
    type Ident = u64;
    type Sink = ToySink<'a>;
    type Source = ToySource<'a>;
    type Error = InvalidEncoding;

    fn sink(&'a mut self, capacity: usize) -> ToySink<'a> {
        ToySink {
            store: &mut self.0,
            bytes: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    fn source(&'a self, id: &Self::Ident) -> Option<Self::Source> {
        self.0.get(id).map(|vec| ToySource { bytes: &vec[..] })
    }
}
