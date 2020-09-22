// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::marker::PhantomData;

use canonical::{ByteSink, ByteSource, Canon, Store};
use canonical_derive::Canon;
use wasmi;

struct CanonImports<S>(S);

impl<S: Store> wasmi::ImportResolver for CanonImports<S> {
    fn resolve_func(
        &self,
        _module_name: &str,
        field_name: &str,
        signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        match field_name {
            "b_get" => {
                panic!("b_get {:?}", &signature);
            }
            _ => panic!("yoo"),
        }
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
#[derive(Canon, Debug)]
pub struct Wasm<State: Module, S: Store> {
    state: State,
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

#[derive(Debug)]
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

#[derive(Debug)]
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
    State: Canon<S> + Module + core::fmt::Debug,
    S: Store,
{
    pub fn new(state: State) -> Self {
        Wasm {
            state,
            _marker: PhantomData,
        }
    }

    pub fn query<A, R>(
        &self,
        query: &Query<A, R>,
        store: S,
    ) -> Result<Result<R, S::Error>, wasmi::Error>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let imports = CanonImports(store.clone());
        let module = wasmi::Module::from_buffer(State::BYTECODE)?;

        let instance =
            wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let mut sink = ByteSink::new(&mut mem[..], store.clone());
                    // Write State and arguments into memory
                    Canon::<S>::write(&self.state, &mut sink)?;
                    Canon::<S>::write(query.args(), &mut sink)
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
                .with_direct_access(|mem| {
                    let mut source = ByteSource::new(&mem[..], store.clone());
                    R::read(&mut source)
                })
                .map_err(Into::into),
            _ => todo!("no memory"),
        })
    }

    pub fn transact<A, R>(
        &mut self,
        transaction: &Transaction<A, R>,
        store: S,
    ) -> Result<Result<R, S::Error>, wasmi::Error>
    where
        A: Canon<S>,
        R: Canon<S>,
    {
        let imports = CanonImports(store.clone());
        let module = wasmi::Module::from_buffer(State::BYTECODE)?;
        let instance = wasmi::ModuleInstance::new(&module, &imports)
            .expect("Failed to instantiate module")
            .assert_no_start();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref
                    .with_direct_access_mut(|mem| {
                        let mut sink = ByteSink::new(mem, store.clone());
                        // First we write State into memory
                        Canon::<S>::write(&self.state, &mut sink)?;
                        // then the arguments, as bytes
                        let res =
                            Canon::<S>::write(transaction.args(), &mut sink);
                        println!("before transaction {:x?}", &mem[0..32]);
                        res
                    })
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
                        println!("after transaction {:x?}", &mem[0..32]);
                        let mut source =
                            ByteSource::new(&mem[..], store.clone());
                        self.state = State::read(&mut source)?;
                        R::read(&mut source)
                    })
                    .map_err(Into::into))
            }
            _ => todo!("no memory"),
        }
    }
}
