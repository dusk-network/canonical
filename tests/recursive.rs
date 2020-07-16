use canon_derive::Canon;
use canonical::{Canon, Handle, Store};

mod toy_store;
use toy_store::ToyStore;

use std::mem;

#[derive(Canon)]
enum Stack<T, S>
where
    S: Store,
{
    Empty,
    Node { value: T, prev: Handle<Self, S> },
}

impl<T, S> Stack<T, S>
where
    S: Store,
    T: Canon,
{
    fn new() -> Self {
        Stack::Empty
    }

    fn push(&mut self, t: T) -> Result<(), <S as Store>::Error> {
        let root = mem::replace(self, Stack::Empty);
        *self = Stack::Node {
            value: t,
            prev: Handle::new(root),
        };
        Ok(())
    }

    fn pop(&mut self) -> Result<Option<T>, <S as Store>::Error> {
        let root = mem::replace(self, Stack::Empty);
        match root {
            Stack::Empty => Ok(None),
            Stack::Node { value, prev } => {
                *self = prev.resolve()?;
                Ok(Some(value))
            }
        }
    }
}

#[test]
fn trivial() {
    let mut store = ToyStore::new();

    let mut list = Stack::<_, ToyStore>::new();

    list.push(8u64).unwrap();

    let id = store.put(&mut list).unwrap();

    let mut restored = store.get::<Stack<u64, ToyStore>>(&id).unwrap().unwrap();

    assert_eq!(restored.pop().unwrap(), Some(8))
}

#[test]
fn multiple() {
    type Int = u8;

    let n: Int = 4;

    let mut store = ToyStore::new();

    let mut list = Stack::<_, ToyStore>::new();

    for i in 0..n {
        list.push(i).unwrap();
    }

    let id = store.put(&mut list).unwrap();

    let mut restored = store.get::<Stack<Int, ToyStore>>(&id).unwrap().unwrap();

    for i in 0..n {
        let i = n - i - 1;
        assert_eq!(restored.pop().unwrap(), Some(i))
    }

    assert_eq!(restored.pop().unwrap(), None)
}
