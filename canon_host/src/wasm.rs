// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::marker::PhantomData;

use canonical::{Canon, CanonError, Store};
use canonical_derive::Canon;
use wasmi;

struct CanonImports;

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

struct Externals;

impl wasmi::Externals for Externals {
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        todo!("a")
    }
}

pub trait Module {
    const BYTECODE: &'static [u8];
}

pub struct Query<A, R> {
    args: A,
    _return: PhantomData<R>,
}

impl<A, R> Query<A, R> {
    pub fn new(args: A) -> Self {
        Query {
            args,
            _return: PhantomData,
        }
    }

    pub fn args(&self) -> &A {
        &self.args
    }
}

pub struct Transaction<A, R> {
    args: A,
    _return: PhantomData<R>,
}

impl<A, R> Transaction<A, R> {
    pub fn new(args: A) -> Self {
        Transaction {
            args,
            _return: PhantomData,
        }
    }

    pub fn args(&self) -> &A {
        &self.args
    }
}

impl<State, S> Wasm<State, S>
where
    State: Canon<S> + Module,
    S: Store,
{
    pub fn new(state: State) -> Self {
        Wasm {
            state,
            bytecode: State::BYTECODE.to_vec(),
            _marker: PhantomData,
        }
    }

    pub fn query<A, R>(
        &self,
        query: &Query<A, R>,
    ) -> Result<Result<R, S::Error>, wasmi::Error>
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
                    // First we write State and arguments into memory
                    Canon::<S>::write(&self.state, sink)?;
                    Canon::<S>::write(query.args(), sink)
                })
            }
            _ => todo!("no memory"),
        }
        .expect("todo, error handling");

        // Perform the query call
        match instance.invoke_export(
            "q",
            &[wasmi::RuntimeValue::I32(0)],
            &mut Externals,
        )? {
            _ => (),
        };

        // read return value
        Ok(match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => memref
                .with_direct_access(|mem| R::read(&mut &mem[..]))
                .map_err(Into::into),
            _ => todo!("no memory"),
        })
    }

    pub fn transact<A, R>(
        &mut self,
        transaction: &Transaction<A, R>,
    ) -> Result<Result<R, S::Error>, wasmi::Error>
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
                memref
                    .with_direct_access_mut::<Result<(), CanonError>, _>(
                        |mem| {
                            let sink = &mut &mut mem[..];
                            // First we write State into memory
                            Canon::<S>::write(&self.state, sink)?;
                            // then the arguments, as bytes
                            Canon::<S>::write(transaction.args(), sink)
                        },
                    )
                    .expect("todo, error");

                match instance.invoke_export(
                    "t",
                    &[wasmi::RuntimeValue::I32(0)],
                    &mut Externals,
                )? {
                    _ => (),
                };

                Ok(memref
                    .with_direct_access(|mem| {
                        let source = &mut &mem[..];
                        self.state = State::read(source)?;
                        R::read(source)
                    })
                    .map_err(Into::into))
            }
            _ => todo!("no memory"),
        }
    }
}
