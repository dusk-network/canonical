use canonical::{Canon, Handle, Store};
use canonical_derive::Canon;
use canonical_host::MemStore;

use std::mem;

#[derive(Canon)]
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

    fn push(&mut self, t: T) -> Result<(), S::Error> {
        let root = mem::replace(self, Stack::Empty);
        *self = Stack::Node {
            value: t,
            prev: Handle::<_, S>::new(root)?,
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
    let mut store = MemStore::new();

    let mut list = Stack::new();

    list.push(8u64).unwrap();

    let snap = store.snapshot(&mut list).unwrap();

    let mut restored = snap.restore().unwrap();

    assert_eq!(restored.pop().unwrap(), Some(8))
}

#[test]
fn multiple() {
    let mut store = MemStore::new();

    type Int = u16;

    let n: Int = 16;

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
