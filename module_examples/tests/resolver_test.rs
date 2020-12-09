// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{MemStore, Remote, Wasm};

use resolver::Counter;

use wasmi::{
    Error, Externals, FuncRef, ModuleImportResolver, RuntimeArgs, RuntimeValue,
    Signature, Trap,
};

use canonical_host::MemoryHolder;

#[derive(Clone, Copy)]
struct HostExternals<'a> {
    memory: Option<&'a wasmi::MemoryRef>,
}

impl<'a> MemoryHolder for HostExternals<'a> {
    fn set_memory(&mut self, memory: &wasmi::MemoryRef) {
        self.memory = Some(memory)
    }
}

const FUNC_INDEX: usize = 100;

impl<'a> Externals for HostExternals<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            FUNC_INDEX => {
                let a: u32 = args.nth_checked(0)?;
                let result = a - 2;

                Ok(Some(RuntimeValue::I32(result as i32)))
            }
            _ => panic!("Unimplemented function at {}", index),
        }
    }
}

impl<'a> ModuleImportResolver for HostExternals<'a> {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, Error> {
        match field_name {
            "host_function" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[wasmi::ValueType::I32][..],
                    Some(wasmi::ValueType::I32),
                ),
                FUNC_INDEX,
            )),

            _ => Err(Error::Instantiation(format!(
                "Export {} not found",
                field_name
            ))),
        }
    }
}

#[test]
fn query() {
    let host_externals = HostExternals { memory: None };

    let store = MemStore::new();
    let wasm_counter = Wasm::new(
        Counter::new(99),
        include_bytes!("../modules/resolver/resolver.wasm"),
    );

    let remote = Remote::new(wasm_counter, &store).unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store, host_externals)
            .unwrap(),
        99
    );
}

#[test]
fn resolver_transaction() {
    let host_externals = HostExternals { memory: None };

    let store = MemStore::new();
    let wasm_counter = Wasm::new(
        Counter::new(99),
        include_bytes!("../modules/resolver/resolver.wasm"),
    );
    let mut remote = Remote::new(wasm_counter, &store).unwrap();

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::adjust(-10), store.clone(), host_externals)
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store, host_externals)
            .unwrap(),
        87
    );
}
