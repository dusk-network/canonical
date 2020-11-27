// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

/// Send debug to host
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {
        #[cfg(feature = "host")]
        $crate::_debug(&format!($( $tt )* ));

        #[cfg(not(feature = "host"))]
        {
            use core::fmt::Write;
            let mut msg = $crate::DebugMsg::new();
            write!(msg, $($tt)*).unwrap();
            $crate::_debug(msg.as_str())
        }
    };
}

#[cfg(not(feature = "host"))]
mod hosted {
    extern "C" {
        fn debug(msg: &u8, len: u32);
    }

    #[doc(hidden)]
    pub fn _debug(buf: &str) {
        let bytes: &[u8] = unsafe { core::mem::transmute(buf) };
        let len = bytes.len() as u32;
        unsafe { debug(&bytes[0], len) }
    }
}

#[cfg(not(feature = "host"))]
pub use hosted::_debug;

#[doc(hidden)]
#[cfg(feature = "host")]
pub fn _debug(msg: &str) {
    println!("HOST: {}", msg)
}

use core::fmt::{self, Write};

impl Write for DebugMsg {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len();
        self.buf[self.ofs..self.ofs + len].copy_from_slice(bytes);
        self.ofs += len;
        Ok(())
    }
}

// Message formatting, not meant to be used directly
// but as a macro expoansion of `debug!`
#[doc(hidden)]
pub struct DebugMsg {
    ofs: usize,
    buf: [u8; 1024 * 16],
}

impl Default for DebugMsg {
    #[doc(hidden)]
    fn default() -> Self {
        Self::new()
    }
}

impl DebugMsg {
    #[doc(hidden)]
    pub fn new() -> Self {
        DebugMsg {
            ofs: 0,
            buf: [0u8; 1024 * 16],
        }
    }

    #[doc(hidden)]
    pub fn bytes(&self) -> &[u8] {
        &self.buf[0..self.ofs]
    }

    #[doc(hidden)]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[0..self.ofs])
            .unwrap_or("DebugMsg.as_str failed.")
    }
}
