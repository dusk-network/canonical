// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

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
pub use repr_host::Repr;

mod dry_sink;
mod id;
mod implementations;
mod store;

pub use canon::{Canon, InvalidEncoding};
pub use dry_sink::DrySink;
pub use id::Id32;
pub use store::{ByteSink, ByteSource, IdBuilder, Ident, Sink, Source, Store};
