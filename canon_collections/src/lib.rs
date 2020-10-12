//! Collection of canonical datastructures

#![cfg_attr(not(feature = "host"), no_std)]
#![deny(missing_docs)]

mod stack;
pub use stack::Stack;
