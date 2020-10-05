// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details.

use std::marker::PhantomData;

use canonical::{ByteSink, ByteSource, Canon, Store};
use canonical_derive::Canon;
use wasmi;

const B_GET: usize = 0;
const B_PUT: usize = 1;

struct CanonImports<S>(S);

impl<S: Store> wasmi::ImportResolver for CanonImports<S> {
    fn resolve_func(
        &self,
        _module_name: &str,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        match field_name {
            "b_get" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(&[wasmi::ValueType::I32][..], None),
                B_GET,
            )),
            "b_put" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                    ][..],
                    None,
                ),
                B_PUT,
            )),
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
#[derive(Canon, Debug, Clone)]
pub struct Wasm<State: Module, S: Store> {
    state: State,
    _marker: PhantomData<S>,
}

struct Externals<'a, S> {
    store: &'a S,
    memory: &'a wasmi::MemoryRef,
}

impl<'a, S> Externals<'a, S> {
    fn new(store: &'a S, memory: &'a wasmi::MemoryRef) -> Self {
        Externals { store, memory }
    }
}

impl<'a, S> wasmi::Externals for Externals<'a, S>
where
    S: Store,
    S::Error: wasmi::HostError,
{
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        match index {
            B_GET => {
                if let [wasmi::RuntimeValue::I32(ofs)] = args.as_ref()[..] {
                    let ofs = ofs as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        // read identifier
                        let mut id = S::Ident::default();
                        let id_len = id.as_ref().len();
                        let slice = &mem[ofs..ofs + id_len];
                        id.as_mut().copy_from_slice(slice);

                        self.store.fetch(&id, &mut mem[ofs..])?;
                        Ok(None)
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            B_PUT => {
                if let [wasmi::RuntimeValue::I32(ofs), wasmi::RuntimeValue::I32(len), wasmi::RuntimeValue::I32(ret_addr)] =
                    args.as_ref()[..]
                {
                    let ofs = ofs as usize;
                    let len = len as usize;
                    let ret_addr = ret_addr as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        if let Ok(id) = self.store.put_raw(&mem[ofs..ofs + len])
                        {
                            let id_len = id.as_ref().len();
                            // write id back
                            mem[ret_addr..ret_addr + id_len]
                                .copy_from_slice(id.as_ref());
                        }
                        Ok(None)
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            _ => panic!("invalid index"),
        }
    }
}

/// Helper trait for wasm bytecode
///
/// TODO: remove this in favor of code being stored in the Wasm wrapper itself.
pub trait Module {
    /// The wasm bytecode associated with this type
    const BYTECODE: &'static [u8];
}

/// Represents the type of a query
#[derive(Debug)]
pub struct Query<A, R> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<R>,
}

impl<A, R> Query<A, R> {
    /// Construct a new query with provided arguments
    pub fn new(args: A) -> Self {
        Query {
            args,
            _return: PhantomData,
        }
    }

    /// Returns a reference to the arguments of a query
    pub fn args(&self) -> &A {
        &self.args
    }
}

/// Represents the type of a transaction
#[derive(Debug)]
pub struct Transaction<A, R> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<R>,
}

impl<A, R> Transaction<A, R> {
    /// Create a new transaction
    pub fn new(args: A) -> Self {
        Transaction {
            args,
            _return: PhantomData,
        }
    }

    /// Returns a reference to the transactions arguments
    pub fn args(&self) -> &A {
        &self.args
    }
}

impl<State, S> Wasm<State, S>
where
    State: Canon<S> + Module + core::fmt::Debug,
    S: Store,
    S::Error: wasmi::HostError,
{
    /// Creates a new Wasm wrapper over an initial state.
    pub fn new(state: State) -> Self {
        Wasm {
            state,
            _marker: PhantomData,
        }
    }

    /// Perform the provided query in the wasm module
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

        Ok(match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let mut sink = ByteSink::new(&mut mem[..], store.clone());
                    // Write State and arguments into memory
                    Canon::<S>::write(&self.state, &mut sink)?;
                    Canon::<S>::write(query.args(), &mut sink)
                })?;

                let mut externals = Externals::new(&store, &memref);

                // Perform the query call
                match instance.invoke_export(
                    "q",
                    &[wasmi::RuntimeValue::I32(0)],
                    &mut externals,
                )? {
                    _ => (),
                };

                memref.with_direct_access_mut(|mem| {
                    // read and return value returned from the invoked query

                    let mut source = ByteSource::new(&mem[..], store.clone());
                    R::read(&mut source)
                })
            }
            _ => panic!("no memory"),
        })
    }

    /// Perform the provided transaction in the wasm module
    pub fn transact<A, R>(
        &mut self,
        transaction: &Transaction<A, R>,
        store: S,
    ) -> Result<Result<R, S::Error>, wasmi::Error>
    where
        A: Canon<S>,
        R: Canon<S>,
        S::Error: wasmi::HostError,
    {
        let imports = CanonImports(store.clone());
        let module = wasmi::Module::from_buffer(State::BYTECODE)?;
        let instance =
            wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();

        Ok(match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let mut sink = ByteSink::new(mem, store.clone());
                    // First we write State into memory
                    Canon::<S>::write(&self.state, &mut sink)?;
                    // then the arguments, as bytes
                    Canon::<S>::write(transaction.args(), &mut sink)
                })?;

                let mut externals = Externals::new(&store, &memref);

                match instance.invoke_export(
                    "t",
                    &[wasmi::RuntimeValue::I32(0)],
                    &mut externals,
                )? {
                    _ => (),
                };

                memref.with_direct_access_mut(|mem| {
                    let mut source = ByteSource::new(&mem[..], store.clone());
                    self.state = State::read(&mut source)?;
                    R::read(&mut source)
                })
            }
            _ => todo!("no memory"),
        })
    }
}
