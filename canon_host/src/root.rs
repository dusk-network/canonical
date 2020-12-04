use canonical::{Canon, Store};

use crate::Transaction;

/// A Store that can perist state.
pub trait Persistent: Store {
    fn set_root(&mut self, root: Self::Ident);
    fn get_root(&self) -> Option<Self::Ident>;
}

// pub trait Transactable<A, R, S>
// where
//     S: Store,
// {
//     fn transact<const ID: u8>(
//         &mut self,
//         t: Transaction<A, R, ID>,
//         store: S,
//     ) -> Result<R, S::Error>;
// }

/// The root of the whole-network state, including
pub struct Root<State, S>
where
    S: Store,
{
    state: State,
    store: S,
}

pub trait Apply<A, R, S, const ID: u8>
where
    S: Store,
{
    fn apply(&mut self, args: A) -> Result<R, S::Error>;
}

impl<State, S> Root<State, S>
where
    State: Default + Canon<S>,
    S: Persistent,
{
    /// Creates a new root from a persistent store
    pub fn new(store: S) -> Result<Self, S::Error> {
        let state = match store.get_root() {
            Some(ref root) => store.get(root)?,
            None => Default::default(),
        };
        Ok(Root { state, store })
    }

    /// Apply a transaction to the state
    pub fn apply<A, R, const ID: u8>(
        &mut self,
        t: Transaction<A, R, ID>,
        store: S,
    ) -> Result<R, S::Error>
    where
        State: Apply<A, R, S, ID>,
    {
        self.state.apply(t.into_args())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{DiskStore, Remote, Transaction, Wasm};

    use canonical::{Canon, Store};
    use canonical_derive::Canon;

    use counter::Counter;

    type ContractId = usize;

    const ADD_CONTRACT: u8 = 0;
    const EXECUTE_CONTRACT_QUERY: u8 = 1;

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
        ) -> Transaction<Remote<S>, ContractId, ADD_CONTRACT> {
            Transaction::new(contract)
        }

        fn execute_contract_query(
            contract: Remote<S>,
        ) -> Transaction<Remote<S>, ContractId, ADD_CONTRACT> {
            Transaction::new(contract)
        }
    }

    impl<S> Apply<Remote<S>, usize, S, ADD_CONTRACT> for TestState<S>
    where
        S: Store,
    {
        fn apply(&mut self, remote: Remote<S>) -> Result<usize, S::Error> {
            let id = self.contracts.len();
            self.contracts.push(remote);
            Ok(id)
        }
    }

    #[test]
    fn create_root() {
        let dir = tempfile::tempdir().unwrap();
        let store = DiskStore::new(dir.path()).unwrap();
        let mut state: Root<TestState<DiskStore>, _> =
            Root::new(store.clone()).unwrap();

        let counter = Counter::new(13);

        let wasm_counter = Wasm::new(
            // unlucky number to not get too lucky in testing
            Counter::new(13),
            include_bytes!(
                "../../module_examples/modules/counter/counter.wasm"
            ),
        );

        // hide counter behind a remote to erase the type
        let remote = Remote::new(wasm_counter, store.clone()).unwrap();
        let transaction = TestState::<DiskStore>::add_contract(remote);
        let id = state.apply(transaction, store.clone()).unwrap();

        let counter_query = Counter::read_value();

        assert_eq!(id, 0);
    }
}
