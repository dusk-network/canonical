// #![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_generics)]
mod canon;
mod handle;
mod implementations;
mod store;

pub use canon::{Canon, ConstantLength, EncodedLength, InvalidEncoding};
pub use handle::Handle;
pub use store::{Sink, Source, Store};
