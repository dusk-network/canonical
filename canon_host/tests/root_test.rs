// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![feature(min_const_generics)]

use canonical_host::{
    wasm, Apply, CastMut, Execute, MemStore as MS, Query, Remote, Transaction,
};

use canonical::{Canon, Store};
use canonical_derive::Canon;

use counter::{self, Counter};
use delegate::{self, Delegator};

type ContractAddr = u64;

// transactions
const ADD_CONTRACT: u8 = 0;
const APPLY_CONTRACT_TRANSACTION: u8 = 1;

// queries
const EXECUTE_CONTRACT_QUERY: u8 = 0;

#[derive(Clone, Canon, Default)]
struct TestState<S: Store> {
    opcount: u64,
    contracts: Vec<Remote<S>>,
}

impl<S> TestState<S>
where
    S: Store,
{
    fn add_contract(
        contract: Remote<S>,
    ) -> Transaction<Self, Remote<S>, ContractAddr, ADD_CONTRACT> {
        Transaction::new(contract)
    }

    fn contract_query<Over, A, R, const ID: u8>(
        &self,
        addr: ContractAddr,
        query: Query<Over, A, R, ID>,
    ) -> Query<
        Self,
        (ContractAddr, Query<Over, A, R, ID>),
        R,
        EXECUTE_CONTRACT_QUERY,
    > {
        Query::new((addr, query))
    }
}

impl<S> Apply<Self, Remote<S>, ContractAddr, S, ADD_CONTRACT> for TestState<S>
where
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<Self, Remote<S>, ContractAddr, ADD_CONTRACT>,
    ) -> Result<ContractAddr, S::Error> {
        let id = self.contracts.len();
        self.contracts.push(transaction.into_args());
        Ok(id as ContractAddr)
    }
}

impl<ContractState, A, R, S, const ID: u8>
    Apply<
        Self,
        (ContractAddr, Transaction<ContractState, A, R, ID>),
        R,
        S,
        APPLY_CONTRACT_TRANSACTION,
    > for TestState<S>
where
    ContractState: Canon<S> + Apply<ContractState, A, R, S, ID>,
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<
            Self,
            (ContractAddr, Transaction<ContractState, A, R, ID>),
            R,
            APPLY_CONTRACT_TRANSACTION,
        >,
    ) -> Result<R, S::Error> {
        let (id, transaction) = transaction.into_args();
        let mut cast_mut: CastMut<ContractState, _> =
            self.contracts[id as usize].cast_mut()?;
        let result = cast_mut.apply(transaction)?;
        Ok(result)
    }
}

impl<ContractState, A, R, S, const ID: u8>
    Execute<
        Self,
        (ContractAddr, Query<ContractState, A, R, ID>),
        R,
        S,
        EXECUTE_CONTRACT_QUERY,
    > for TestState<S>
where
    ContractState: Canon<S> + Execute<ContractState, A, R, S, ID>,
    S: Store,
{
    fn execute(
        &self,
        query: Query<
            Self,
            (ContractAddr, Query<ContractState, A, R, ID>),
            R,
            EXECUTE_CONTRACT_QUERY,
        >,
    ) -> Result<R, S::Error> {
        let (id, query) = query.into_args();
        let cast: ContractState = self.contracts[id as usize].cast()?;
        cast.execute(query)
    }
}

#[test]
fn create_root() {
    let store = MS::new();
    let mut state = TestState::default();

    let counter = Counter::new(13);

    let wasm_counter = wasm::Wasm::new(
        // unlucky number to not get too lucky in testing
        counter,
        store.clone(),
        include_bytes!("../../module_examples/modules/counter/counter.wasm"),
    );

    // hide counter behind a remote to erase the type
    let remote = Remote::new(wasm_counter, store.clone()).unwrap();

    let transaction = TestState::<MS>::add_contract(remote);
    let id = state.apply(transaction).unwrap();

    assert_eq!(id, 0);

    let query = Query::<
        _,
        (
            ContractAddr,
            Query<
                wasm::Wasm<Counter, MS>,
                Query<Counter, (), i32, { counter::READ_VALUE }>,
                i32,
                { wasm::WASM_QUERY },
            >,
        ),
        i32,
        EXECUTE_CONTRACT_QUERY,
    >::new((id, Query::new(Counter::read_value())));

    let result = state.execute(query).unwrap();
    assert_eq!(result, 13);

    // increment
    let transaction = Transaction::<
        _,
        (
            ContractAddr,
            Transaction<
                wasm::Wasm<Counter, MS>,
                Transaction<Counter, (), (), { counter::INCREMENT }>,
                (),
                { wasm::WASM_TRANSACTION },
            >,
        ),
        (),
        APPLY_CONTRACT_TRANSACTION,
    >::new((id, Transaction::new(Counter::increment())));

    state.apply(transaction).unwrap();

    // should have updated

    let query = Query::<
        _,
        (
            ContractAddr,
            Query<
                wasm::Wasm<Counter, MS>,
                Query<Counter, (), i32, { counter::READ_VALUE }>,
                i32,
                { wasm::WASM_QUERY },
            >,
        ),
        i32,
        EXECUTE_CONTRACT_QUERY,
    >::new((id, Query::new(Counter::read_value())));

    let result = state.execute(query).unwrap();
    assert_eq!(result, 14);
}

#[test]
fn delegate_calls() {
    let store = MS::new();
    let mut state = TestState::default();

    let counter_a = wasm::Wasm::new(
        // unlucky number to not get too lucky in testing
        Counter::new(1234),
        store.clone(),
        include_bytes!("../../module_examples/modules/counter/counter.wasm"),
    );

    let counter_b = wasm::Wasm::new(
        // unlucky number to not get too lucky in testing
        Counter::new(4321),
        store.clone(),
        include_bytes!("../../module_examples/modules/counter/counter.wasm"),
    );

    let delegator = wasm::Wasm::new(
        // unlucky number to not get too lucky in testing
        Delegator::new(),
        store.clone(),
        include_bytes!("../../module_examples/modules/delegate/delegate.wasm"),
    );

    let remote_a = Remote::new(counter_a, store.clone()).unwrap();
    let remote_b = Remote::new(counter_b, store.clone()).unwrap();
    let remote_c = Remote::new(delegator, store.clone()).unwrap();

    let transaction_a = TestState::<MS>::add_contract(remote_a);
    let transaction_b = TestState::<MS>::add_contract(remote_b);
    let transaction_c = TestState::<MS>::add_contract(remote_c);

    let id_a = state.apply(transaction_a).unwrap();
    let id_b = state.apply(transaction_b).unwrap();
    let id_c = state.apply(transaction_c).unwrap();

    // network state is setup here

    let delegated_query_a = Delegator::delegate_read_value(id_a);
    let delegated_query_b = Delegator::delegate_read_value(id_b);

    let wasm_delegated_query_a = wasm::Wasm::query(delegated_query_a);
    let wasm_delegated_query_b = wasm::Wasm::query(delegated_query_b);

    let state_query_a = state.contract_query(id_c, wasm_delegated_query_a);
    let state_query_b = state.contract_query(id_c, wasm_delegated_query_b);

    assert_eq!(state.execute(state_query_a).unwrap(), 1234);
    assert_eq!(state.execute(state_query_b).unwrap(), 4321);
}
