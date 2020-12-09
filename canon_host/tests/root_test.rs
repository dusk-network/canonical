#![feature(min_const_generics)]

use canonical_host::{
    wasm, Apply, CastMut, Execute, MemStore, Query, Remote, Root, Transaction,
};

use canonical::{Canon, Store};
use canonical_derive::Canon;

use counter::{Counter, INCREMENT, READ_VALUE};

type ContractAddr = usize;

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
}

impl<S> Apply<Self, Remote<S>, usize, S, ADD_CONTRACT> for TestState<S>
where
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<Self, Remote<S>, usize, ADD_CONTRACT>,
    ) -> Result<usize, S::Error> {
        let id = self.contracts.len();
        self.contracts.push(transaction.into_args());
        Ok(id)
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
            self.contracts[id].cast_mut()?;
        let result = cast_mut.apply(transaction)?;
        cast_mut.commit()?;
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
        let cast: ContractState = self.contracts[id].cast()?;
        cast.execute(query)
    }
}

#[test]
fn create_root() {
    let store = MemStore::new();
    let mut state = Root::default();

    let counter = Counter::new(13);

    let wasm_counter = wasm::Wasm::new(
        // unlucky number to not get too lucky in testing
        counter,
        store.clone(),
        include_bytes!("../../module_examples/modules/counter/counter.wasm"),
    );

    // hide counter behind a remote to erase the type
    let remote = Remote::new(wasm_counter, store.clone()).unwrap();

    let transaction = TestState::<MemStore>::add_contract(remote);
    let id = state.apply(transaction).unwrap();

    assert_eq!(id, 0);

    let query = Query::<
        _,
        (
            ContractAddr,
            Query<
                wasm::Wasm<Counter, MemStore>,
                Query<Counter, (), i32, READ_VALUE>,
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
                wasm::Wasm<Counter, MemStore>,
                Transaction<Counter, (), (), INCREMENT>,
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
                wasm::Wasm<Counter, MemStore>,
                Query<Counter, (), i32, READ_VALUE>,
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
