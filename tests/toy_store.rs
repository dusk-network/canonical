use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use canon::{Canon, InvalidEncoding, Sink, Source, Store};

#[derive(Default)]
pub struct ToyStore(HashMap<u64, Vec<u8>>);

pub struct ToySink<'a> {
    store: &'a mut HashMap<u64, Vec<u8>>,
    bytes: Vec<u8>,
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

    fn request_bytes(&mut self, num_bytes: usize) -> &mut [u8] {
        let len = self.bytes.len();
        self.bytes.resize_with(len + num_bytes, || 0);
        &mut self.bytes[len..]
    }

    fn provide_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes)
    }

    fn fin(self) -> Self::Ident {
        let mut hasher = DefaultHasher::new();
        self.bytes.hash(&mut hasher);
        let id = hasher.finish();
        self.store.insert(id, self.bytes);
        id
    }
}

impl<'a> Store<'a> for ToyStore {
    type Ident = u64;
    type Sink = ToySink<'a>;
    type Source = ToySource<'a>;
    type Error = InvalidEncoding;

    fn sink(&'a mut self) -> ToySink<'a> {
        ToySink {
            store: &mut self.0,
            bytes: vec![],
        }
    }

    fn source(&'a self, id: &Self::Ident) -> Option<Self::Source> {
        self.0.get(id).map(|vec| ToySource { bytes: &vec[..] })
    }
}

#[test]
fn sink() {
    let a: u64 = 38382;

    let mut store = ToyStore::new();

    let mut sink = store.sink();
    a.write(&mut sink);
    let id = sink.fin();

    assert_eq!(
        u64::read(&mut store.source(&id).expect("missing value"))
            .expect("invalid encoding"),
        a
    );
}
