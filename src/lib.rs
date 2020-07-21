//! Canonical, a no_std, host-allocating serialization library
#![no_std]
#![allow(incomplete_features)]
#![warn(missing_docs)]
#![feature(const_generics)]
mod canon;
mod handle;
mod implementations;
mod snapshot;
mod store;

pub use canon::{Canon, CanonError};
pub use handle::Handle;
pub use snapshot::{Snap, Snapshot};
pub use store::{Ident, Sink, Source, Store};
