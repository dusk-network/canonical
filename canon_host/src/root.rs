use canonical::{Canon, Store};

use crate::{Apply, Execute, Query, Transaction};

/// A Store that can perist state.
pub trait Persistent: Store {
    fn set_root(&mut self, root: Self::Ident);
    fn get_root(&self) -> Option<Self::Ident>;
}

/// The root of the whole-network state, including
pub struct Root<State, S>
where
    S: Store,
{
    #[allow(unused)]
    state: State,
    #[allow(unused)]
    store: S,
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
}

impl<A, R, State, S, const ID: u8> Apply<A, R, S, ID> for Root<State, S>
where
    State: Apply<A, R, S, ID>,
    S: Store,
{
    fn apply(
        &mut self,
        transaction: Transaction<A, R, ID>,
    ) -> Result<R, S::Error> {
        self.state.apply(transaction)
    }
}

impl<State, A, R, S, const ID: u8> Execute<State, A, R, S, ID>
    for Root<State, S>
where
    S: Store,
    State: Execute<State, A, R, S, ID>,
{
    fn execute(&self, query: Query<State, A, R, ID>) -> Result<R, S::Error> {
        self.state.execute(query)
    }
}
