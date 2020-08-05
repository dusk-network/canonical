use canonical::{Canon, CanonError, Handle, Store, VoidStore};
use canonical_derive::Canon;
use canonical_host::MemStore;

use std::mem;

#[derive(Clone, Canon, Debug)]
enum Stack<T, S: Store> {
    Empty,
    Node { value: T, prev: Handle<Self, S> },
}

impl<T, S> Stack<T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn new() -> Self {
        Stack::Empty
    }

    fn push(&mut self, t: T) -> Result<(), CanonError<S>> {
        let root = mem::replace(self, Stack::Empty);
        *self = Stack::Node {
            value: t,
            prev: Handle::<_, S>::new(root)?,
        };
        Ok(())
    }

    fn pop(&mut self) -> Result<Option<T>, CanonError<S>> {
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
fn multiple() {
    type Int = u64;

    let n: Int = 1024;

    let mut list = Stack::<_, VoidStore>::new();

    for i in 0..n {
        let _ = list.push(i);
    }

    for i in 0..n {
        let i = n - i - 1;
        assert_eq!(list.pop().unwrap(), Some(i))
    }

    assert_eq!(list.pop().unwrap(), None)
}

#[test]
fn multiple_restored() {
    let store = MemStore::new();

    type Int = u8;

    let n: Int = 128;

    let mut list = Stack::new();

    for i in 0..n {
        list.push(i).unwrap();
    }

    let snap = store.snapshot(&mut list).unwrap();
    let mut restored = snap.restore().unwrap();

    for i in 0..n {
        let i = n - i - 1;
        assert_eq!(restored.pop().unwrap(), Some(i))
    }

    assert_eq!(restored.pop().unwrap(), None)
}
