// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::marker::PhantomData;

use wasmi;

use canonical::{Canon, CanonError, Store};
use canonical_derive::Canon;
use canonical_host::{MemStore, Remote};

use counter::Counter;

const BUF_SIZE: usize = 1024 * 4;

struct CanonImports;

impl wasmi::ImportResolver for CanonImports {
    fn resolve_func(
        &self,
        _module_name: &str,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_global(
        &self,
        module_name: &str,
        field_name: &str,
        descriptor: &wasmi::GlobalDescriptor,
    ) -> Result<wasmi::GlobalRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_memory(
        &self,
        module_name: &str,
        field_name: &str,
        descriptor: &wasmi::MemoryDescriptor,
    ) -> Result<wasmi::MemoryRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_table(
        &self,
        module_name: &str,
        field_name: &str,
        descriptor: &wasmi::TableDescriptor,
    ) -> Result<wasmi::TableRef, wasmi::Error> {
        unimplemented!()
    }
}

/// A type with a corresponding wasm module
#[derive(Canon)]
pub struct Wasm<State, S> {
    state: State,
    bytecode: [u8; 0],
    _marker: PhantomData<S>,
}

impl<State, S: Store> Wasm<State, S> {
    fn query(&self, query: &[u8], ret: &mut [u8]) -> Result<(), wasmi::Error> {
        let module = wasmi::Module::from_buffer(&self.bytecode)?;
        let instance = wasmi::ModuleInstance::new(&module, &CanonImports)
            .expect("Failed to instantiate module")
            .assert_no_start();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => panic!(),
            _ => todo!("no memory"),
        }
    }
}

impl<S: Store> Wasm<Counter, S> {
    fn read_state(&self) -> Result<u32, CanonError<S::Error>> {
        let mut result = [0u8; BUF_SIZE];
        self.query(&[], &mut result);
        <u32 as Canon<S>>::read(&mut &result[..])
    }
}

#[test]
fn wasm_contracts() {
    let mut world = Vec::new();

    let bytecode = include_bytes!("../examples/counter/counter.wasm");

    let store = MemStore::new();

    let wasm_counter = Wasm {
        bytecode: [], // bytecode.to_vec()
        state: Counter::new(99),
        _marker: PhantomData as PhantomData<MemStore>,
    };

    world.push(Remote::new(wasm_counter, &store).unwrap());

    assert_eq!(
        world[0]
            .query::<Wasm<Counter, MemStore>>()
            .unwrap()
            .read_state()
            .unwrap(),
        99
    );

    // let mut transaction =
    //     world[0].transact::<Wasm<Counter, MemStore>>().unwrap();

    // transaction.adjust(33).unwrap();
    // transaction.adjust(-1).unwrap();

    // transaction.commit().unwrap();

    // assert_eq!(
    //     world[0]
    //         .query::<Wasm<Counter, MemStore>>()
    //         .unwrap()
    //         .read_state()
    //         .unwrap(),
    //     32
    // );
}
