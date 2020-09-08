// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::marker::PhantomData;

use wasmi;

use canonical::{Canon, Store};
use canonical_derive::Canon;
use canonical_host::{MemStore, Remote};

use counter::Counter;

const BUF_SIZE: usize = 1024 * 4;

struct CanonImports;

type Error = Box<dyn std::error::Error>;

impl wasmi::ImportResolver for CanonImports {
    fn resolve_func(
        &self,
        _module_name: &str,
        _field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_global(
        &self,
        _module_name: &str,
        _field_name: &str,
        _descriptor: &wasmi::GlobalDescriptor,
    ) -> Result<wasmi::GlobalRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_memory(
        &self,
        _module_name: &str,
        _field_name: &str,
        _descriptor: &wasmi::MemoryDescriptor,
    ) -> Result<wasmi::MemoryRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_table(
        &self,
        _module_name: &str,
        _field_name: &str,
        _descriptor: &wasmi::TableDescriptor,
    ) -> Result<wasmi::TableRef, wasmi::Error> {
        unimplemented!()
    }
}

/// A type with a corresponding wasm module
#[derive(Canon)]
pub struct Wasm<State, S: Store> {
    state: State,
    bytecode: Vec<u8>,
    _marker: PhantomData<S>,
}

impl<S, State> wasmi::Externals for Wasm<State, S>
where
    State: Canon<S>,
    S: Store,
{
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        //
        todo!()
    }
}

impl<State, S> Wasm<State, S>
where
    State: Canon<S>,
    S: Store,
{
    fn query<A, R>(&self, args: &A) -> Result<R, Error>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let module = wasmi::Module::from_buffer(&self.bytecode)?;
        let instance = wasmi::ModuleInstance::new(&module, &CanonImports)?
            .assert_no_start();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let sink = &mut &mut mem[..];
                    // First we write State into memory
                    Canon::<S>::write(&self.state, sink);
                    // then the arguments, as bytes
                    Canon::<S>::write(args, sink);
                });
            }
            _ => todo!("no memory"),
        }

        // TODO: perform the query call

        // read return value
        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => memref
                .with_direct_access(|mem| {
                    let source = &mut &mem[..];
                    let ret = R::read(source).expect("todo, error overhaul");
                    Ok(ret)
                }),
            _ => todo!("no memory"),
        }
    }

    pub fn transact<A, R>(
        &mut self,
        args: &A,
    ) -> Result<R, Box<dyn std::error::Error>>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let module = wasmi::Module::from_buffer(&self.bytecode)?;
        let instance = wasmi::ModuleInstance::new(&module, &CanonImports)
            .expect("Failed to instantiate module")
            .assert_no_start();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let sink = &mut &mut mem[..];
                    // First we write State into memory
                    Canon::<S>::write(&self.state, sink);
                    // then the arguments, as bytes
                    Canon::<S>::write(args, sink);
                });

                match instance.invoke_export("t", &[], self)? {
                    _ => (),
                };

                // read return value

                memref.with_direct_access_mut(|mem| {
                    let source = &mut &mem[..];
                    let ret = R::read(source).expect("todo, error overhaul");
                    Ok(ret)
                })
            }
            _ => todo!("no memory"),
        }
    }
}

impl<S: Store> Wasm<Counter, S> {
    fn read_state(&self) -> Result<u32, Error> {
        self.query(&())
    }

    fn adjust(&mut self, by: i32) -> Result<(), Error> {
        self.transact(&by)
    }
}

#[test]
fn wasm_contracts() {
    let mut world = Vec::new();

    let bytecode = include_bytes!("../examples/counter/counter.wasm");

    let store = MemStore::new();

    let wasm_counter = Wasm {
        bytecode: bytecode.to_vec(),
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

    let mut transaction =
        world[0].transact::<Wasm<Counter, MemStore>>().unwrap();

    transaction.adjust(-33).unwrap();
    transaction.commit();

    assert_eq!(
        world[0]
            .query::<Wasm<Counter, MemStore>>()
            .unwrap()
            .read_state()
            .unwrap(),
        66
    );
}
