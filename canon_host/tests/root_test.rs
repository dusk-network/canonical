#![feature(min_const_generics)]

use canonical_host::{
    wasm, Apply, DiskStore, Execute, Query, Remote, Root, Transaction,
};

use canonical::{Canon, Store};
use canonical_derive::Canon;

use counter::{Counter, READ_VALUE};

type ContractAddr = usize;

const ADD_CONTRACT: u8 = 0;
const APPLY_CONTRACT_TRANSACTION: u8 = 1;
const EXECUTE_CONTRACT_QUERY: u8 = 2;

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
    ) -> Transaction<Remote<S>, ContractAddr, ADD_CONTRACT> {
        Transaction::new(contract)
    }
}

impl<S> Apply<Remote<S>, usize, S, ADD_CONTRACT> for TestState<S>
where
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<Remote<S>, usize, ADD_CONTRACT>,
    ) -> Result<usize, S::Error> {
        let id = self.contracts.len();
        self.contracts.push(transaction.into_args());
        Ok(id)
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
    let dir = tempfile::tempdir().unwrap();
    let store = DiskStore::new(dir.path()).unwrap();
    let mut state: Root<TestState<DiskStore>, _> =
        Root::new(store.clone()).unwrap();

    let counter = Counter::new(13);

    let wasm_counter = wasm::Wasm::new(
        // unlucky number to not get too lucky in testing
        counter,
        store.clone(),
        include_bytes!("../../module_examples/modules/counter/counter.wasm"),
    );

    // hide counter behind a remote to erase the type
    let remote = Remote::new(wasm_counter, store.clone()).unwrap();

    let transaction = TestState::<DiskStore>::add_contract(remote);
    let id = state.apply(transaction).unwrap();

    assert_eq!(id, 0);

    let counter_query: Query<
        wasm::Wasm<Counter, DiskStore>,
        Query<Counter, (), i32, READ_VALUE>,
        i32,
        { wasm::WASM_QUERY },
    > = Query::new(Counter::read_value());

    let wrapped_query = crate::Query::<
        _,
        (
            ContractAddr,
            crate::Query<
                wasm::Wasm<Counter, DiskStore>,
                Query<Counter, (), i32, READ_VALUE>,
                i32,
                { wasm::WASM_QUERY },
            >,
        ),
        i32,
        EXECUTE_CONTRACT_QUERY,
    >::new((id, counter_query));

    let result = state.execute(wrapped_query).unwrap();

    assert_eq!(result, 13);
}
