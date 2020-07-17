// #![cfg_attr(not(feature = "std"), no_std)]
#![allow(incomplete_features)]
#![feature(const_generics)]
mod canon;
mod handle;
mod implementations;
mod snapshot;
mod store;

pub use canon::{Canon, CanonError, ConstantLength, EncodedLength};
pub use handle::Handle;
pub use snapshot::{Snap, Snapshot};
pub use store::{Sink, Source, Store};
