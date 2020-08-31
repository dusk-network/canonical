// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

#![feature(never_type)]
mod mem_store;
pub use mem_store::MemStore;

mod remote;
pub use remote::{Remote, Cast, Query, Transact};
