// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

//! Canonical, a no_std, host-allocating serialization library
#![cfg_attr(feature = "bridge", no_std)]
#![allow(incomplete_features)]
#![warn(missing_docs)]
#![feature(const_generics)]
#![feature(lang_items)]
#![feature(never_type)]

mod canon;

#[cfg(feature = "bridge")]
mod bridge;
#[cfg(feature = "bridge")]
mod handle_bridge;
#[cfg(feature = "bridge")]
pub use handle_bridge::Handle;

#[cfg(not(feature = "bridge"))]
mod handle_host;
#[cfg(not(feature = "bridge"))]
pub use handle_host::Handle;

#[cfg(feature = "bridge")]
pub use bridge::BridgeStore;

mod implementations;
mod store;

pub use canon::{Canon, CanonError};

pub use store::{Ident, Sink, Source, Store, VoidStore};
