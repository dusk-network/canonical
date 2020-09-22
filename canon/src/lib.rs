// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(feature = "hosted", no_std)]
#![allow(incomplete_features)]
#![warn(missing_docs)]
#![feature(const_generics)]
#![feature(lang_items)]

mod canon;

#[cfg(feature = "hosted")]
mod bridge;
#[cfg(feature = "hosted")]
mod handle_hosted;
#[cfg(feature = "hosted")]
pub use handle_hosted::Handle;

#[cfg(feature = "host")]
mod handle_host;
#[cfg(feature = "host")]
pub use handle_host::Handle;

#[cfg(feature = "hosted")]
pub use bridge::BridgeStore;

mod implementations;
mod store;

pub use canon::{Canon, InvalidEncoding};

pub use store::{ByteSink, ByteSource, Ident, Sink, Source, Store};
