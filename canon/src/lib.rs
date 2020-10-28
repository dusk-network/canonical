// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(not(feature = "host"), no_std)]
#![feature(min_const_generics)]
#![deny(missing_docs)]

mod canon;
mod repr;

pub use repr::Repr;

#[cfg(not(feature = "host"))]
mod bridge;

#[cfg(not(feature = "host"))]
pub use bridge::BridgeStore;

mod cow;
mod dry_sink;
mod id;
mod implementations;
mod store;

pub use canon::{Canon, InvalidEncoding};
pub use cow::{Cow, CowMut};
pub use dry_sink::DrySink;
pub use id::Id32;
pub use store::{ByteSink, ByteSource, IdBuilder, Ident, Sink, Source, Store};
