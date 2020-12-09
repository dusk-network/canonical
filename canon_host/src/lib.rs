// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! The host library responsible for running and connecting modules

#![deny(missing_docs)]
#![feature(min_const_generics)]

mod mem_store;
pub use mem_store::{MemError, MemStore};

mod disk_store;
pub use disk_store::{DiskError, DiskStore};

mod root;
pub use root::Root;

mod remote;
pub use remote::{Apply, Execute, Query, Remote, Transaction};

/// Module containing wasm related modules.
/// FIXME, move this to its own crate?
pub mod wasm;
pub use wasm::{ExternalsResolver, Signal, Wasm};

/// FIXME: this does not really belong here
pub mod common;
