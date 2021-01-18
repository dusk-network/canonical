// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt;
use std::sync::Arc;

use parking_lot::RwLock;

use appendix::Index;

use canonical::{
    ByteSink, Canon, DrySink, Id32, InvalidEncoding, Sink, Source, Store,
};
use canonical_derive::Canon;

use std::fs::{create_dir, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use tempfile::tempdir;

pub struct DiskStoreInner {
    index: Index<Id32, u64>,
    data: File,
    data_path: PathBuf,
    data_offset: u64,
}

impl Default for DiskStoreInner {
    fn default() -> Self {
        DiskStoreInner::new(
            tempdir()
                .expect("Unable to access to the temporary folder")
                .into_path(),
        )
        .expect("Unable to create a disk store")
    }
}

/// An in-memory store implemented with a hashmap
#[derive(Default, Clone)]
pub struct DiskStore(Arc<RwLock<DiskStoreInner>>);

impl DiskStore {
    /// Creates a new DiskStore
    pub fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        Ok(DiskStore(Arc::new(RwLock::new(DiskStoreInner::new(path)?))))
    }
}

impl DiskStoreInner {
    /// Create a new DiskStore
    pub fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let dir = path.into();
        if !dir.exists() {
            create_dir(&dir)?;
        }

        let index_dir = dir.join("index");
        if !index_dir.exists() {
            create_dir(&index_dir)?;
        }

        let index = Index::new(&index_dir)?;
        let data_path = dir.join("data");

        let mut data = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&data_path)?;

        let data_offset = data.metadata()?.len();
        data.seek(SeekFrom::End(0))?;

        Ok(Self {
            index,
            data_path,
            data,
            data_offset,
        })
    }
}

struct DiskSink<S> {
    bytes: Vec<u8>,
    store: S,
}

struct DiskSource<'a, S> {
    bytes: &'a [u8],
    offset: usize,
    store: S,
}

#[derive(Canon, Debug, Clone, PartialEq)]
/// Errors that can happen using the DiskStore.
pub enum DiskError {
    /// Value missing in the store
    MissingValue,
    /// Invalid data
    InvalidEncoding,
    /// Generic IO Error, TODO: improve
    Io,
}

impl From<io::Error> for DiskError {
    fn from(_err: io::Error) -> DiskError {
        DiskError::Io
    }
}

impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingValue => write!(f, "Missing Value"),
            Self::InvalidEncoding => write!(f, "InvalidEncoding"),
            Self::Io => write!(f, "Generic IO Error"),
        }
    }
}

impl From<InvalidEncoding> for DiskError {
    fn from(_: InvalidEncoding) -> Self {
        DiskError::InvalidEncoding
    }
}

impl Store for DiskStore {
    type Ident = Id32;
    type Error = DiskError;

    fn fetch(
        &self,
        id: &Self::Ident,
        into: &mut [u8],
    ) -> Result<(), Self::Error> {
        match self.0.read().index.get(id)? {
            Some(offset) => {
                let mut file = File::open(&self.0.read().data_path)?;
                file.seek(SeekFrom::Start(*offset))?;
                file.read_exact(into)?;
                Ok(())
            }
            None => Err(DiskError::MissingValue),
        }
    }

    fn get<T: Canon<Self>>(&self, id: &Self::Ident) -> Result<T, Self::Error> {
        match self.0.read().index.get(id)? {
            Some(offset) => {
                let mut file = File::open(&self.0.read().data_path)?;
                file.seek(SeekFrom::Start(*offset))?;

                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;

                let mut source = DiskSource {
                    bytes: &bytes[..],
                    offset: *offset as usize,
                    store: self.clone(),
                };
                T::read(&mut source)
            }
            None => Err(DiskError::MissingValue),
        }
    }

    fn put<T: Canon<Self>>(&self, t: &T) -> Result<Self::Ident, Self::Error> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = ByteSink::new(&mut bytes, self.clone());
        Canon::<Self>::write(t, &mut sink)?;
        let ident = sink.fin();

        if self
            .0
            .read()
            .index
            .insert(ident, self.0.read().data_offset)?
        {
            // value already present
            Ok(ident)
        } else {
            self.0.write().data.write_all(&bytes)?;
            self.0.write().data_offset += bytes.len() as u64;

            // need to add flush
            self.0.write().data.flush()?;
            self.0.write().index.flush()?;

            Ok(ident)
        }
    }

    fn put_raw(&self, bytes: &[u8]) -> Result<Self::Ident, Self::Error> {
        let mut sink = DrySink::<Self>::new();
        sink.copy_bytes(bytes);
        let ident = sink.fin();

        if self
            .0
            .read()
            .index
            .insert(ident, self.0.read().data_offset)?
        {
            // value already present
            Ok(ident)
        } else {
            self.0.write().data.write_all(&bytes)?;
            self.0.write().data_offset += bytes.len() as u64;

            // need to add flush
            self.0.write().data.flush()?;
            self.0.write().index.flush()?;

            Ok(ident)
        }
    }
}

impl<S: Store> Sink<S> for DiskSink<S> {
    fn copy_bytes(&mut self, bytes: &[u8]) {
        let ofs = self.bytes.len();
        self.bytes.resize_with(ofs + bytes.len(), || 0);
        self.bytes[ofs..].clone_from_slice(bytes)
    }

    fn recur<T: Canon<S>>(&self, t: &T) -> Result<S::Ident, S::Error> {
        self.store.put(t)
    }

    fn fin(self) -> S::Ident {
        todo!("this is unreasonable")
    }
}

impl<'a, S> Source<S> for DiskSource<'a, S>
where
    S: Store,
{
    fn read_bytes(&mut self, n: usize) -> &[u8] {
        let ofs = self.offset;
        self.offset += n;
        &self.bytes[ofs..self.offset]
    }

    fn store(&self) -> &S {
        &self.store
    }
}
