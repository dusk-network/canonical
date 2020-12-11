// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical_host::{MemStore, Remote, Wasm};

use resolver::Counter;

use wasmi::{
    Error, Externals, FuncRef, ModuleImportResolver, RuntimeArgs, RuntimeValue,
    Signature, Trap, TrapKind,
};

use canonical_host::MemoryHolder;

#[derive(Clone, Debug)]
struct HostExternals {
    memory: Option<wasmi::MemoryRef>,
}

impl MemoryHolder for HostExternals {
    fn set_memory(&mut self, memory: wasmi::MemoryRef) {
        self.memory = Some(memory);
    }
    fn memory(&self) -> Result<wasmi::MemoryRef, wasmi::Trap> {
        self.memory
            .to_owned()
            .ok_or_else(|| Trap::new(TrapKind::ElemUninitialized))
    }
}

const FUNC_INDEX: usize = 100;

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            FUNC_INDEX => {
                if let [wasmi::RuntimeValue::I32(ofs)] = args.as_ref()[..] {
                    let ofs = ofs as usize;
                    self.memory()?.with_direct_access_mut(|mem| {
                        let mut bytes = [0u8; 4];
                        bytes.copy_from_slice(&mem[ofs..ofs + 4]);
                        let result = i32::from_le_bytes(bytes);
                        Ok(Some(RuntimeValue::I32(result as i32)))
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            _ => panic!("Unimplemented function at {}", index),
        }
    }
}

impl<'a> ModuleImportResolver for HostExternals {
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
    let wasm_resolver = Wasm::new(
        Counter::new(99),
        include_bytes!("../modules/resolver/resolver.wasm"),
    );

    let remote = Remote::new(wasm_resolver, &store).unwrap();

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
    let wasm_resolver = Wasm::new(
        Counter::new(99),
        include_bytes!("../modules/resolver/resolver.wasm"),
    );
    let mut remote = Remote::new(wasm_resolver, &store).unwrap();

    let mut cast = remote.cast_mut::<Wasm<Counter, MemStore>>().unwrap();
    cast.transact(&Counter::adjust(-10), store.clone(), host_externals.clone())
        .unwrap();
    cast.commit().unwrap();

    assert_eq!(
        remote
            .cast::<Wasm<Counter, MemStore>>()
            .unwrap()
            .query(&Counter::read_value(), store, host_externals)
            .unwrap(),
        89
    );
}
