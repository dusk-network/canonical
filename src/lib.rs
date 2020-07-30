//! Canonical, a no_std, host-allocating serialization library
#![no_std]
#![allow(incomplete_features)]
#![warn(missing_docs)]
#![feature(const_generics)]
mod bridge;
mod canon;

mod handle_bridge;

mod implementations;
mod snapshot;
mod store;

pub use canon::{Canon, CanonError};
pub use handle_bridge::Handle;
pub use snapshot::Snapshot;
pub use store::{Ident, Sink, Source, Store};
