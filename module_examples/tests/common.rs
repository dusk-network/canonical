// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use wasmi::{
    Error, Externals, FuncRef, ModuleImportResolver, RuntimeArgs, RuntimeValue,
    Signature, Trap,
};

use canonical_host::MemoryHolder;

#[derive(Clone, Copy)]
pub struct HostExternals {}

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        Ok(None)
    }
}

impl ModuleImportResolver for HostExternals {
    fn resolve_func(
        &self,
        _field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, Error> {
        unimplemented!();
    }
}

impl MemoryHolder for HostExternals {
    fn set_memory(&mut self, _memory: &wasmi::MemoryRef) {}
}

pub fn no_externals() -> HostExternals {
    HostExternals {}
}
