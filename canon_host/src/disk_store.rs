// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;

use parking_lot::RwLock;

use appendix::Index;

use canonical::{Canon, CanonError, Id, IdBuilder, Sink};

use std::fs::{create_dir, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub struct DiskStoreInner {
    #[allow(unused)] // FIXME
    index: Index<Id, u64>,
    #[allow(unused)] // FIXME
    data: File,
    #[allow(unused)] // FIXME
    data_path: PathBuf,
    #[allow(unused)] // FIXME
    data_offset: u64,
}

/// An in-memory store implemented with a hashmap
#[derive(Clone)]
pub struct DiskStore(Arc<RwLock<DiskStoreInner>>);

impl DiskStore {
    /// Creates a new DiskStore
    pub fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        Ok(DiskStore(Arc::new(RwLock::new(DiskStoreInner::new(path)?))))
    }
}

impl DiskStoreInner {
    /// Create a new DiskStoreInner
    fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
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

impl DiskStore {
    #[allow(unused)] // FIXME
    fn fetch(&self, id: &Id, into: &mut [u8]) -> Result<(), CanonError> {
        match self.0.read().index.get(id).map_err(CanonError::from_e)? {
            Some(offset) => {
                let mut file = File::open(&self.0.read().data_path)
                    .map_err(CanonError::from_e)?;
                file.seek(SeekFrom::Start(*offset))
                    .map_err(CanonError::from_e)?;
                file.read_exact(into).map_err(CanonError::from_e)?;
                Ok(())
            }
            None => Err(CanonError::NotFound),
        }
    }

    #[allow(unused)] // FIXME
    fn get<T: Canon>(&self, _id: &Id) -> Result<T, io::Error> {
        // match self.0.read().index.get(id).map_err(CanonError::from_e)? {
        //     Some(offset) => {
        //         let mut file = File::open(&self.0.read().data_path)
        //             .map_err(CanonError::from_e)?;
        //         file.seek(SeekFrom::Start(*offset))
        //             .map_err(CanonError::from_e)?;

        //         let mut bytes = Vec::new();
        //         file.read_to_end(&mut bytes).map_err(CanonError::from_e)?;

        //         let mut source = Source {
        //             bytes: &bytes[..],
        //             offset: *offset as usize,
        //         };
        //         Ok(T::read(&mut source).unwrap())
        //     }
        //     None => Err(CanonError::NotFound),
        // }
        todo!()
    }

    #[allow(unused)] // FIXME
    fn put<T: Canon>(&self, t: &T) -> Result<Id, io::Error> {
        let len = t.encoded_len();
        let mut bytes = Vec::with_capacity(len);
        bytes.resize_with(len, || 0);

        let mut sink = Sink::new(&mut bytes);
        Canon::write(t, &mut sink);
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

    #[allow(unused)] // FIXME
    fn put_raw(&self, bytes: &[u8]) -> Result<Id, io::Error> {
        let ident = Id::new(bytes);

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
