// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

//! The host library responsible for running and connecting modules

#![deny(missing_docs)]

mod mem_store;
pub use mem_store::MemStore;

mod remote;
pub use remote::Remote;

mod wasm;
pub use wasm::{Module, Query, Transaction, Wasm};
