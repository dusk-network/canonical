// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Collection of canonical datastructures

#![cfg_attr(not(feature = "host"), no_std)]
#![deny(missing_docs)]

mod stack;
pub use stack::Stack;
