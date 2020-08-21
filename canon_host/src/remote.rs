use canonical::{Canon, Store};

pub struct Remote<S: Store>(S::Ident);

impl<S: Store> Remote<S> {
    pub fn new<T: Canon<S>>(_t: T, _store: &S) -> Self {
        unimplemented!()
    }
}
