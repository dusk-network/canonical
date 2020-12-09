// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt;
use std::marker::PhantomData;

use crate::{Apply, Execute, Query, Transaction};
use canonical::{ByteSink, ByteSource, Canon, Store};
use canonical_derive::Canon;

/// Query id for executing queries over Wasm modules.
pub const WASM_QUERY: u8 = 0;

/// Transaction id for applying transactions over Wasm modules.
pub const WASM_TRANSACTION: u8 = 0;

const GET: usize = 0;
const PUT: usize = 1;
const SIG: usize = 2;
const DBG: usize = 999;

#[derive(Canon, Clone, Debug, PartialEq)]
/// A panic signal that can be sent from a module.
pub enum Signal {
    /// Signal originated as a panic
    Panic(String),
    /// Signal originated as an error
    Error(String),
}

/// Super trait that requires both wasmi::Externals and
/// wasmi::ModuleImportResolver
pub trait ExternalsResolver:
    wasmi::Externals + wasmi::ModuleImportResolver
{
}

impl<T: wasmi::Externals + wasmi::ModuleImportResolver> ExternalsResolver
    for T
{
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Signal::Panic(s) => write!(f, "Panic {}", s),
            Signal::Error(s) => write!(f, "Error {}", s),
        }
    }
}

impl Signal {
    /// Create a new signal
    pub fn panic<S: Into<String>>(s: S) -> Self {
        Signal::Panic(s.into())
    }
}

impl wasmi::HostError for Signal {}

// We do this rather complicated dance to sneak our custom error out of a rather
// uwieldly nested set of enums.
impl From<wasmi::Error> for Signal {
    fn from(err: wasmi::Error) -> Self {
        match err {
            wasmi::Error::Trap(ref trap) => match trap.kind() {
                wasmi::TrapKind::Host(h) => match h.downcast_ref::<Signal>() {
                    Some(s) => s.clone(),
                    None => todo!(),
                },
                _ => Signal::Error(format!("{}", err)),
            },
            _ => Signal::Error(format!("{}", err)),
        }
    }
}

struct CanonImports<S>(S);

impl<S> wasmi::ModuleImportResolver for CanonImports<S>
where
    S: Store,
    S::Error: From<Signal>,
{
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &wasmi::Signature,
    ) -> Result<wasmi::FuncRef, wasmi::Error> {
        match field_name {
            "get" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(&[wasmi::ValueType::I32][..], None),
                GET,
            )),
            "put" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                    ][..],
                    None,
                ),
                PUT,
            )),
            "sig" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[wasmi::ValueType::I32, wasmi::ValueType::I32][..],
                    None,
                ),
                SIG,
            )),
            "debug" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[wasmi::ValueType::I32, wasmi::ValueType::I32][..],
                    None,
                ),
                DBG,
            )),
            _ => panic!("yoo {}", field_name),
        }
    }

    fn resolve_global(
        &self,
        _field_name: &str,
        _descriptor: &wasmi::GlobalDescriptor,
    ) -> Result<wasmi::GlobalRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_memory(
        &self,
        _field_name: &str,
        _descriptor: &wasmi::MemoryDescriptor,
    ) -> Result<wasmi::MemoryRef, wasmi::Error> {
        unimplemented!()
    }

    fn resolve_table(
        &self,
        _field_name: &str,
        _descriptor: &wasmi::TableDescriptor,
    ) -> Result<wasmi::TableRef, wasmi::Error> {
        unimplemented!()
    }
}

/// A type with a corresponding wasm module
#[derive(Canon, Clone)]
pub struct Wasm<State, S: Store> {
    state: State,
    store: S,
    bytecode: Vec<u8>,
    _marker: PhantomData<S>,
}

impl<S, State> core::fmt::Debug for Wasm<State, S>
where
    State: core::fmt::Debug,
    S: Store,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Wasm {:?}", self.state)
    }
}

struct Externals<'a, S, E> {
    store: &'a S,
    memory: &'a wasmi::MemoryRef,
    ext: E,
}

impl<'a, S, E> Externals<'a, S, E> {
    fn new(store: &'a S, memory: &'a wasmi::MemoryRef, ext: E) -> Self {
        Externals { store, memory, ext }
    }
}

impl<'a, S, I> wasmi::Externals for Externals<'a, S, I>
where
    S: Store,
    S::Error: wasmi::HostError,
    S::Error: From<Signal>,
    I: wasmi::Externals,
{
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        match index {
            GET => {
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
            PUT => {
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
            SIG => {
                if let [wasmi::RuntimeValue::I32(ofs), wasmi::RuntimeValue::I32(len)] =
                    args.as_ref()[..]
                {
                    let ofs = ofs as usize;
                    let len = len as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        let bytes = &mem[ofs..ofs + len];
                        let string =
                            String::from_utf8_lossy(&bytes).to_string();
                        let signal = Signal::panic(string);
                        Err(wasmi::Trap::new(wasmi::TrapKind::Host(Box::new(
                            signal,
                        ))))
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            DBG => {
                if let [wasmi::RuntimeValue::I32(ofs), wasmi::RuntimeValue::I32(len)] =
                    args.as_ref()[..]
                {
                    let ofs = ofs as usize;
                    let len = len as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        let bytes = &mem[ofs..ofs + len];
                        let string =
                            String::from_utf8_lossy(&bytes).to_string();
                        println!("HOSTED: {}", string);
                        Ok(None)
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            _ => self.ext.invoke_index(index, args),
        }
    }
}

impl<State, S> Wasm<State, S>
where
    State: Canon<S>,
    S: Store,
    S::Error: wasmi::HostError,
    S::Error: From<Signal>,
{
    /// Creates a new Wasm wrapper over an initial state.
    pub fn new(state: State, store: S, bytecode: &[u8]) -> Self {
        Wasm {
            state,
            store,
            bytecode: Vec::from(bytecode),
            _marker: PhantomData,
        }
    }
}

impl<State, A, R, S, const ID: u8>
    Execute<Self, Query<State, A, R, ID>, R, S, WASM_QUERY> for Wasm<State, S>
where
    A: Canon<S>,
    R: Canon<S>,
    State: Canon<S>,
    S: Store,
    S::Error: From<Signal>,
    S::Error: wasmi::HostError,
    S::Error: From<wasmi::Error>,
{
    fn execute(
        &self,
        query: Query<Self, Query<State, A, R, ID>, R, WASM_QUERY>,
    ) -> Result<R, S::Error> {
        let resolver = crate::common::HostExternals {};

        let mut imports = wasmi::ImportsBuilder::default();
        let canon_module = CanonImports(self.store.clone());
        imports.push_resolver("canon", &canon_module);

        let module = wasmi::Module::from_buffer(&self.bytecode)?;

        let instance =
            wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();

        let inner_query = query.into_args();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let mut sink = ByteSink::new(&mut mem[..], &self.store);
                    // Write State, method_id and arguments into memory
                    Canon::<S>::write(&self.state, &mut sink)?;
                    Canon::<S>::write(&ID, &mut sink)?;
                    Canon::<S>::write(inner_query.args(), &mut sink)
                })?;

                let mut externals =
                    Externals::new(&self.store, &memref, resolver);

                // Perform the query call
                instance.invoke_export(
                    "q",
                    &[wasmi::RuntimeValue::I32(0)],
                    &mut externals,
                )?;

                memref.with_direct_access_mut(|mem| {
                    // read and return value returned from the invoked query

                    let mut source = ByteSource::new(&mem[..], &self.store);
                    R::read(&mut source)
                })
            }
            _ => panic!("no memory"),
        }
    }
}

impl<State, A, R, S, const ID: u8>
    Apply<Self, Transaction<State, A, R, ID>, R, S, WASM_TRANSACTION>
    for Wasm<State, S>
where
    A: Canon<S>,
    R: Canon<S>,
    State: Canon<S>,
    S: Store,
    S::Error: From<Signal>,
    S::Error: wasmi::HostError,
    S::Error: From<wasmi::Error>,
{
    fn apply(
        &mut self,
        transaction: Transaction<
            Self,
            Transaction<State, A, R, ID>,
            R,
            WASM_TRANSACTION,
        >,
    ) -> Result<R, S::Error> {
        let resolver = crate::common::HostExternals {};

        let mut imports = wasmi::ImportsBuilder::default();
        let canon_module = CanonImports(self.store.clone());
        imports.push_resolver("canon", &canon_module);
        imports.push_resolver("env", &resolver);

        let inner_transaction = transaction.into_args();

        let module = wasmi::Module::from_buffer(&self.bytecode)?;
        let instance =
            wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();

        match instance.export_by_name("memory") {
            Some(wasmi::ExternVal::Memory(memref)) => {
                memref.with_direct_access_mut(|mem| {
                    let mut sink = ByteSink::new(mem, &self.store);
                    // First we write State into memory
                    Canon::<S>::write(&self.state, &mut sink)?;
                    // then the method id
                    Canon::<S>::write(&ID, &mut sink)?;
                    // then the arguments, as bytes
                    Canon::<S>::write(inner_transaction.args(), &mut sink)
                })?;

                let mut externals =
                    Externals::new(&self.store, &memref, resolver);

                instance.invoke_export(
                    "t",
                    &[wasmi::RuntimeValue::I32(0)],
                    &mut externals,
                )?;

                memref.with_direct_access_mut(|mem| {
                    let mut source = ByteSource::new(&mem[..], &self.store);
                    self.state = State::read(&mut source)?;
                    let res = R::read(&mut source)?;
                    Ok(res)
                })
            }
            _ => todo!("no memory"),
        }
    }
}
