// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(not(feature = "host"), no_std)]
#![deny(missing_docs)]

mod canon;

#[cfg(not(feature = "host"))]
mod bridge;
#[cfg(not(feature = "host"))]
mod repr_hosted;
#[cfg(not(feature = "host"))]
pub use bridge::BridgeStore;
#[cfg(not(feature = "host"))]
pub use repr_hosted::Repr;

#[cfg(feature = "host")]
mod repr_host;
#[cfg(feature = "host")]
pub use repr_host::{Repr, ValMut};

mod dry_sink;
mod id;
mod implementations;
mod store;

pub use canon::{Canon, InvalidEncoding};
pub use dry_sink::DrySink;
pub use id::Id32;
pub use store::{ByteSink, ByteSource, IdBuilder, Ident, Sink, Source, Store};
