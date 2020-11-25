// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt;
use std::marker::PhantomData;

use canonical::{ByteSink, ByteSource, Canon, Store};
use canonical_derive::Canon;
use dusk_bls12_381::BlsScalar;
use dusk_plonk::prelude::*;
use poseidon252::sponge::sponge::sponge_hash;
use schnorr::single_key::{PublicKey, Signature};
use wasmi;

const GET: usize = 0;
const PUT: usize = 1;
const SIG: usize = 2;
const P_HASH: usize = 3;
const VERIFY_BID_PROOF: usize = 4;
const VERIFY: usize = 5;
const VERIFY_SIG: usize = 6;
const DBG: usize = 999;

#[derive(Canon, Clone, Debug, PartialEq)]
/// A panic signal that can be sent from a module.
pub enum Signal {
    /// Signal originated as a panic
    Panic(String),
    /// Signal originated as an error
    Error(String),
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
                _ => Signal::Error(String::from(format!("{}", err))),
            },
            _ => Signal::Error(String::from(format!("{}", err))),
        }
    }
}

struct CanonImports<S>(S);

impl<S> wasmi::ImportResolver for CanonImports<S>
where
    S: Store,
    S::Error: From<Signal>,
{
    fn resolve_func(
        &self,
        _module_name: &str,
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
            "p_hash" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                    ][..],
                    None,
                ),
                P_HASH,
            )),
            "verify" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[wasmi::ValueType::I32, wasmi::ValueType::I32][..],
                    None,
                ),
                VERIFY,
            )),
            "verify_bid_proof" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                    ][..],
                    None,
                ),
                VERIFY_BID_PROOF,
            )),
            "verify_sig" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                    ][..],
                    None,
                ),
                VERIFY_SIG,
            )),
            "debug" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[wasmi::ValueType::I32, wasmi::ValueType::I32][..],
                    None,
                ),
                DBG,
            )),
            _ => panic!("Unknown host fn {}", field_name),
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
#[derive(Canon, Clone)]
pub struct Wasm<State, S: Store> {
    state: State,
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
    S::Error: From<Signal>,
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
            P_HASH => {
                if let [wasmi::RuntimeValue::I32(ofs), wasmi::RuntimeValue::I32(len), wasmi::RuntimeValue::I32(ret_addr)] =
                    args.as_ref()[..]
                {
                    let ofs = ofs as usize;
                    let len = len as usize;
                    let ret_addr = ret_addr as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        let bytes = &mem[ofs..ofs + len];
                        // Chunk bytes to BlsSclar byte-size
                        let inp: Vec<BlsScalar> = bytes
                            .chunks(32usize)
                            .map(|scalar_bytes| {
                                let mut array = [0u8; 32];
                                array.copy_from_slice(&scalar_bytes[..]);
                                BlsScalar::from_bytes(&array).unwrap()
                            })
                            .collect();
                        let result = sponge_hash(&inp);
                        mem[ret_addr..ret_addr + 32]
                            .copy_from_slice(&result.to_bytes()[..]);
                        // Read Scalars from Chunks
                        Ok(None)
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            VERIFY => {
                if let [wasmi::RuntimeValue::I32(ofs), wasmi::RuntimeValue::I32(ret_addr)] =
                    args.as_ref()[..]
                {
                    let ofs = ofs as usize;
                    let len = Proof::serialised_size() as usize;
                    let ret_addr = ret_addr as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        let bytes = &mem[ofs..ofs + len];
                        // Chunk bytes to BlsSclar byte-size
                        let proof = Proof::from_bytes(&bytes[..]).unwrap();

                        mem[ret_addr] = 1u8;
                        // Read Scalars from Chunks
                        Ok(None)
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            VERIFY_SIG => {
                if let [wasmi::RuntimeValue::I32(pk), wasmi::RuntimeValue::I32(sig), wasmi::RuntimeValue::I32(msg), wasmi::RuntimeValue::I32(ret_addr)] =
                    args.as_ref()[..]
                {
                    let pk = pk as usize;
                    let sig = sig as usize;
                    let msg = msg as usize;
                    let ret_addr = ret_addr as usize;
                    self.memory.with_direct_access_mut(|mem| {
                        // Build Pk
                        let mut bytes32 = [0u8; 32];
                        let mut bytes64 = [0u8; 64];
                        bytes32[0..32].copy_from_slice(&mem[pk..pk + 32]);
                        let pk = PublicKey::from_bytes(&bytes32).unwrap();
                        // Build Sig
                        bytes64[0..64].copy_from_slice(&mem[sig..sig + 64]);
                        let sig = Signature::from_bytes(&bytes64).unwrap();
                        // Build Msg
                        bytes32[0..32].copy_from_slice(&mem[msg..msg + 32]);
                        let msg = BlsScalar::from_bytes(&bytes32).unwrap();
                        // Perform the signature verification
                        match sig.verify(&pk, msg) {
                            Ok(()) => mem[ret_addr] = 1u8,
                            _ => mem[ret_addr] = 0u8,
                        };
                        Ok(None)
                    })
                } else {
                    todo!("error out for wrong argument types")
                }
            }
            /*VERIFY_PROOF => {
                if let [wasmi::RuntimeValue::I32(ofs_proof), wasmi::RuntimeValue::I32(ofs_pp), wasmi::RuntimeValue::I32(len_pp), wasmi::RuntimeValue::I32(ofs_vk), wasmi::RuntimeValue::I32(len_vk), wasmi::RuntimeValue::I32(ret_addr)] =
                    args.as_ref()[..]
                {
                    let ofs_proof = ofs_proof as usize;
                    let ofs_pp = ofs_pp as usize;
                    let len_pp = len_pp as usize;
                    let ofs_vk = ofs_vk as usize;
                    let len_vk = len_vk as usize;
                    let ret_addr = ret_addr as usize;
                    // Generate Proof
                    self.memory.with_direct_access_mut(|mem| {
                        // Generate Proof
                        let proof_bytes = &mem
                            [ofs_proof..ofs_proof + Proof::serialised_size()];
                        let proof = Proof::from_bytes(&proof_bytes).unwrap();
                        // Generate Pub Params
                        let pp_bytes = &mem[ofs_pp..ofs_pp + len_pp];
                        let pub_params =
                            PublicParameters::from_bytes(&pp_bytes).unwrap();
                        // Generate VK
                        let vk_bytes = &mem[ofs_vk..ofs_vk + len_vk];
                        let vk = VerifierKey::from_bytes(&vk_bytes).unwrap();
                        // Verify the proof and return true or false.
                        let mut verifier = Verifier::new(b"Test");
                        verifier.verifier_key = vk;
                        verifier.verify(&proof, &op_key, &pi);
                    });
                    Ok(None)
                } else {
                    todo!("Handle this")
                }
            }*/
            _ => panic!("invalid index"),
        }
    }
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
    State: Canon<S>,
    S: Store,
    S::Error: wasmi::HostError,
    S::Error: From<Signal>,
{
    /// Creates a new Wasm wrapper over an initial state.
    pub fn new(state: State, bytecode: &[u8]) -> Self {
        Wasm {
            state,
            bytecode: Vec::from(bytecode),
            _marker: PhantomData,
        }
    }

    /// Perform the provided query in the wasm module
    pub fn query<A, R>(
        &self,
        query: &Query<A, R>,
        store: S,
    ) -> Result<R, S::Error>
    where
        A: Canon<S>,
        R: Canon<S>,
        S::Error: From<wasmi::Error>,
    {
        let imports = CanonImports(store.clone());
        let module = wasmi::Module::from_buffer(&self.bytecode)?;

        let instance =
            wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();

        match instance.export_by_name("memory") {
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
        }
    }

    /// Perform the provided transaction in the wasm module
    pub fn transact<A, R>(
        &mut self,
        transaction: &Transaction<A, R>,
        store: S,
    ) -> Result<R, S::Error>
    where
        A: Canon<S>,
        R: Canon<S>,
        S::Error: wasmi::HostError,
        S::Error: From<Signal>,
        S::Error: From<wasmi::Error>,
    {
        let imports = CanonImports(store.clone());
        let module = wasmi::Module::from_buffer(&self.bytecode)?;
        let instance =
            wasmi::ModuleInstance::new(&module, &imports)?.assert_no_start();

        match instance.export_by_name("memory") {
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
                    let res = R::read(&mut source)?;
                    Ok(res)
                })
            }
            _ => todo!("no memory"),
        }
    }
}
