use canonical::{Canon, Handle, Store};
use canonical_derive::Canon;

#[derive(Clone, Canon, Debug)]
pub enum Stack<T, S: Store> {
    Empty,
    Node { value: T, prev: Handle<Self, S> },
}

impl<T, S> Stack<T, S>
where
    S: Store,
    T: Canon<S> + Clone,
{
    pub fn new() -> Self {
        Stack::Empty
    }

    pub fn push(&mut self, t: T) -> Result<(), S::Error> {
        let root = core::mem::replace(self, Stack::Empty);
        *self = Stack::Node {
            value: t,
            prev: Handle::<_, S>::new(root)?,
        };
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Option<T>, S::Error> {
        let root = core::mem::replace(self, Stack::Empty);
        match root {
            Stack::Empty => Ok(None),
            Stack::Node { value, prev } => {
                *self = prev.restore()?;
                Ok(Some(value))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use canonical_host::MemStore;

    #[test]
    fn multiple() {
        type Int = u64;

        let n: Int = 1024;

        let mut list = Stack::<_, MemStore>::new();

        for i in 0..n {
            list.push(i).unwrap();
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
            list.push(i).unwrap()
        }

        let mut handle = Handle::new(list).unwrap();

        handle.commit(&store).unwrap();

        let mut restored = handle.restore().unwrap();

        for i in 0..n {
            let i = n - i - 1;
            assert_eq!(restored.pop().unwrap(), Some(i))
        }

        assert_eq!(restored.pop().unwrap(), None)
    }

    #[test]
    fn multiple_restored_tuples() {
        fn exec<F, T>(tuple: F)
        where
            F: Fn(u8) -> T,
            T: core::fmt::Debug + Canon<MemStore> + Clone + Eq,
        {
            let store = MemStore::new();

            type Int = u8;

            let n: Int = 128;

            let mut list = Stack::new();

            for i in 0..n {
                list.push(tuple(i));
            }

            let mut handle = Handle::new(list);

            handle.commit(&store).unwrap();

            let mut restored = handle.restore().unwrap();

            for i in (0..n).rev() {
                assert_eq!(restored.pop().unwrap(), Some(tuple(i)))
            }

            assert_eq!(restored.pop().unwrap(), None)
        }

        fn enforce_impl<T>(_t: T)
        where
            T: Canon<MemStore>,
        {
        }

        exec(|i: u8| (i));
        exec(|i: u8| (i, i));
        exec(|i: u8| (i, i, i));
        exec(|i: u8| (i, i, i, i));
        exec(|i: u8| (i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i, i, i, i, i, i));
        exec(|i: u8| (i, i, i, i, i, i, i, i, i, i, i, i));

        enforce_impl((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2));
        enforce_impl((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3));
        enforce_impl((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4));
        enforce_impl((0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5));
    }
}
