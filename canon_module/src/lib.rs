// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg_attr(not(feature = "host"), no_std)]
#![feature(min_const_generics)]

#[cfg(feature = "host")]
mod test_resolver;
#[cfg(feature = "host")]
pub use test_resolver::TestResolver;

mod module;
mod query;
mod transaction;

// Maximum size of transactions
const Q_T_SIZE: usize = 1024 * 64;

#[cfg(feature = "host")]
pub mod wasm;

pub use module::{Apply, CastMut, Execute, Module};
pub use query::{Query, RawQuery};
pub use transaction::{RawTransaction, Transaction};

#[cfg(feature = "host")]
pub use wasm::Signal;
