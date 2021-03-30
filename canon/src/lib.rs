// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(target_arch = "wasm32", no_std)]
#![feature(never_type)]
#![deny(missing_docs)]

extern crate alloc;

mod canon;
mod id;
mod implementations;
mod repr;
mod store;

pub use canon::{Canon, CanonError, EncodeToVec};
pub use id::{Id, IdHash};
pub use repr::{Repr, Val, ValMut};
pub use store::{Sink, Source, Store};
