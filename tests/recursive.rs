use canon::{Canon, Handle, InvalidEncoding, Store};
use canon_derive::Canon;

mod toy_store;
use toy_store::ToyStore;

#[derive(Canon, Debug)]
enum Stack<T, S> {
    Empty,
    Node { value: T, prev: Handle<Self, S> },
}

impl<T, S> Stack<T, S>
where
    S: for<'a> Store<'a>,
{
    fn new() -> Self {
        Stack::Empty
    }

    fn push(&mut self, t: T) -> Result<(), <S as Store>::Error> {
        match self {
            Stack::Empty => {
                *self = Stack::Node {
                    value: t,
                    prev: Handle::new(Stack::Empty),
                };
                unimplemented!();
            }
            _ => unimplemented!(),
        }
    }

    fn pop(&mut self) -> Result<Option<T>, <S as Store>::Error> {
        match self {
            Stack::Empty => Ok(None),
            Stack::Node { value: _, prev: _ } => {
                unimplemented!()
                //*self = prev.into();
                //Ok(Some(value))
            }
        }
    }
}

#[test]
fn linked_list() {
    let mut store = ToyStore::new();

    let mut list = Stack::<_, ToyStore>::new();

    list.push(8u64).unwrap();

    let id = store.put(&mut list);

    let mut restored = store.get::<Stack<u64, ToyStore>>(&id).unwrap().unwrap();

    assert_eq!(restored.pop().unwrap(), Some(8))
}
